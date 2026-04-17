;;; cljvindent.el --- Indent Clojure, ClojureScript, and EDN using a native module -*- lexical-binding: t; -*-

;; Copyright (C) 2026 Your Panagiotis Koromilias
;; SPDX-License-Identifier: Apache-2.0

;; Author: Panagiotis Koromilias
;; Version: 0.1.0
;; Package-Requires: ((emacs "29.1"))
;; URL: https://github.com/narocath/cljvindent-emacs
;; Keywords: tools

;;; Commentary:
;; cljvindent formats Clojure(script) code using a native module.
;; It can format:
;; - the current form at point
;; - the parent form of the current form
;; - the outer parent form of the current form
;; - the top-level form at point
;; - the active region
;; - the whole file

;;; Code:

(require 'thingatpt)
(require 'cljvindent-build)

(declare-function cljvindent--indent-string nil
                  (text base-col enable-logs log-level log-file-output-type))
(declare-function cljvindent--indent-clj-file nil
                  (file enable-logs log-level log-file-output-type))

(defvar cljvindent--module-loaded nil
  "Non-nil once the native module has been loaded.")

(defun cljvindent--module-file ()
  "Return cljvindent module file path."
  (expand-file-name
   (concat cljvindent--module-name module-file-suffix)
   (file-name-directory
    (or load-file-name
        (locate-library "cljvindent")))))

(defun cljvindent--load-module ()
  "Load the native module if present.
Return non-nil on success."
  (unless cljvindent--module-loaded
    (let ((module (cljvindent--module-file)))
      (when (file-exists-p module)
        (load (file-name-sans-extension module) nil 'nomessage)
        (setq cljvindent--module-loaded t))))
  cljvindent--module-loaded)

(defun cljvindent--ensure-module ()
  "Ensure the cljvindent native module exists and is loaded."
  (or (cljvindent--load-module)
      (when cljvindent-auto-build-module
        (when (y-or-n-p "Module cljvindent is missing. Build it now? ")
          (cljvindent-build-module)
          (cljvindent--load-module)))
      (user-error "Module cljvindent is not available")))

;;;###autoload
(defun cljvindent-install-module ()
  "Build and load the cljvindent native module."
  (interactive)
  (let ((was-loaded cljvindent--module-loaded))
    (cljvindent-build-module)
    (cond
     (was-loaded
      (message "Native module rebuilt. Restart Emacs to use the new version"))
     ((cljvindent--load-module)
      (message "Native module cljvindent is ready"))
     (t
      (user-error "Module cljvindent was built but could not be loaded")))))

(defun cljvindent--supported-mode-p ()
  "Return non-nil if the current buffer is in a supported mode."
  (derived-mode-p 'clojure-mode 'clojurescript-mode 'edn-mode))

(defun cljvindent--slice-form-data (start end)
  "Return plist data for the form between START and END."
  (list :start start
        :end end
        :text (buffer-substring-no-properties start end)
        :base-col (save-excursion
                    (goto-char start)
                    (current-column))))

(defun cljvindent--top-level-form-data ()
  "Return plist data for the top-level form at point, or nil."
  (save-excursion
    (condition-case nil
        (progn
          (beginning-of-defun)
          (let ((start (point)))
            (end-of-defun)
            (cljvindent--slice-form-data start (point))))
      (error nil))))

(defun cljvindent--current-form-data ()
  "Return plist data for the current form at point, or nil."
  (save-excursion
    (let ((ppss (syntax-ppss)))
      (unless (or (nth 3 ppss) (nth 4 ppss))
        (condition-case nil
            (cond
             ((looking-at-p "\\s(")
              (let ((start (point))
                    (end (scan-sexps (point) 1)))
                (when end
                  (cljvindent--slice-form-data start end))))
             (t
              (let ((bounds (bounds-of-thing-at-point 'sexp)))
                (when bounds
                  (cljvindent--slice-form-data (car bounds) (cdr bounds))))))
          (error nil))))))

(defun cljvindent--enclosing-form-data (levels)
  "Return plist data for the enclosing form LEVELS up from point, or nil."
  (save-excursion
    (let ((ppss (syntax-ppss)))
      (unless (or (nth 3 ppss) (nth 4 ppss))
        (condition-case nil
            (progn
              (unless (looking-at-p "\\s(")
                (let ((bounds (bounds-of-thing-at-point 'sexp)))
                  (when bounds
                    (goto-char (car bounds)))))
              (dotimes (_ levels)
                (backward-up-list 1))
              (let ((start (point))
                    (end (scan-sexps (point) 1)))
                (when end
                  (cljvindent--slice-form-data start end))))
          (error nil))))))

(defun cljvindent--parent-form-data ()
  "Return plist data for the parent form of the current form, or nil."
  (cljvindent--enclosing-form-data 1))

(defun cljvindent--outer-parent-form-data ()
  "Return plist data for the outer parent form of the current form, or nil."
  (cljvindent--enclosing-form-data 2))

(defun cljvindent--region-form-data ()
  "Return plist data for the active region, or nil."
  (when (use-region-p)
    (cljvindent--slice-form-data (region-beginning) (region-end))))

(defun cljvindent--replace-form-with (form-data formatter-fn)
  "Replace FORM-DATA with the result of calling FORMATTER-FN."
  (unless form-data
    (user-error "No form or region found"))
  (let* ((start (plist-get form-data :start))
         (end (plist-get form-data :end))
         (text (substring-no-properties (plist-get form-data :text)))
         (base-col (plist-get form-data :base-col))
         (raw-result
          (funcall formatter-fn
                   text
                   base-col
                   cljvindent-enable-logs
                   cljvindent-log-level
                   cljvindent-log-file-output-type))
         (result (substring-no-properties raw-result))
         (pos (point))
         (tmp (generate-new-buffer " *cljvindent-replace*")))
    (unwind-protect
        (progn
          (with-current-buffer tmp
            (insert result))
          (with-undo-amalgamate
            (save-excursion
              (save-restriction
                (narrow-to-region start end)
                (replace-buffer-contents tmp)))))
      (kill-buffer tmp))
    (goto-char (min pos (point-max)))
    result))

;;;###autoload
(defun cljvindent-current-form ()
  "Indent the current form at point."
  (interactive)
  (unless (cljvindent--supported-mode-p)
    (user-error "Supported modes: clojure-mode, clojurescript-mode, and edn-mode"))
  (cljvindent--ensure-module)
  (let ((start-time (current-time)))
    (cljvindent--replace-form-with
     (cljvindent--current-form-data)
     #'cljvindent--indent-string)
    (message "cljvindent done in %.3fs"
             (float-time (time-subtract (current-time) start-time)))))

;;;###autoload
(defun cljvindent-top-level-form ()
  "Indent the top-level form at point."
  (interactive)
  (unless (cljvindent--supported-mode-p)
    (user-error "Supported modes: clojure-mode, clojurescript-mode, and edn-mode"))
  (cljvindent--ensure-module)
  (let ((start-time (current-time)))
    (cljvindent--replace-form-with
     (cljvindent--top-level-form-data)
     #'cljvindent--indent-string)
    (message "cljvindent done in %.3fs"
             (float-time (time-subtract (current-time) start-time)))))

;;;###autoload
(defun cljvindent-parent ()
  "Indent the parent form of the current form."
  (interactive)
  (unless (cljvindent--supported-mode-p)
    (user-error "Supported modes: clojure-mode, clojurescript-mode, and edn-mode"))
  (cljvindent--ensure-module)
  (let ((start-time (current-time)))
    (cljvindent--replace-form-with
     (cljvindent--parent-form-data)
     #'cljvindent--indent-string)
    (message "cljvindent done in %.3fs"
             (float-time (time-subtract (current-time) start-time)))))

;;;###autoload
(defun cljvindent-outer-parent ()
  "Indent the outer parent form of the current form."
  (interactive)
  (unless (cljvindent--supported-mode-p)
    (user-error "Supported modes: clojure-mode, clojurescript-mode, and edn-mode"))
  (cljvindent--ensure-module)
  (let ((start-time (current-time)))
    (cljvindent--replace-form-with
     (cljvindent--outer-parent-form-data)
     #'cljvindent--indent-string)
    (message "cljvindent done in %.3fs"
             (float-time (time-subtract (current-time) start-time)))))

;;;###autoload
(defun cljvindent-region ()
  "Indent the active region."
  (interactive)
  (unless (cljvindent--supported-mode-p)
    (user-error "Supported modes: clojure-mode, clojurescript-mode, and edn-mode"))
  (cljvindent--ensure-module)
  (let ((start-time (current-time)))
    (cljvindent--replace-form-with
     (cljvindent--region-form-data)
     #'cljvindent--indent-string)
    (message "cljvindent done in %.3fs"
             (float-time (time-subtract (current-time) start-time)))))

;;;###autoload
(defun cljvindent-whole-buffer ()
  "Indent the whole file of the current buffer."
  (interactive)
  (unless (cljvindent--supported-mode-p)
    (user-error "Supported modes: clojure-mode, clojurescript-mode, and edn-mode"))
  (cljvindent--ensure-module)
  (unless buffer-file-name
    (user-error "Current buffer is not visiting a file"))
  (when (buffer-modified-p)
    (unless (y-or-n-p "Buffer has unsaved changes. Save before indenting? ")
      (user-error "Indenting aborted"))
    (save-buffer))
  (let ((start-time (current-time)))
    (cljvindent--indent-clj-file
     buffer-file-name
     cljvindent-enable-logs
     cljvindent-log-level
     cljvindent-log-file-output-type)
    (revert-buffer :ignore-auto :noconfirm)
    (message "cljvindent done in %.3fs"
             (float-time (time-subtract (current-time) start-time)))))

(provide 'cljvindent)

;;; cljvindent.el ends here
