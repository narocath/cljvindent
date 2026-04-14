;;; cljvindent.el --- Indent Clojure forms using a Rust native module -*- lexical-binding: t; -*-

;; Author: Panagiotis Koromilias
;; Version: 0.1.0
;; Package-Requires: ((emacs "29.1"))
;; URL: https://github.com/narocath/cljvindent
;; Keywords: tools

;;; Commentary:
;; cljvindent formats Clojure(script) code using a Rust native module.
;; It can format:
;; - the current form at point
;; - the parent form at point
;; - the outer parent form at point
;; - the top-level form at point
;; - the active region
;; - the whole file

(require 'subr-x)
(require 'cljvindent-build)

(defconst cljvindent--module-name "clj_vindent_emacs_module")

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
  "Ensure cljvindent module exists and is loaded."
  (interactive)
  (or (cljvindent--load-module)
      (when cljvindent-auto-build-module
        (when (y-or-n-p "Module cljvindent is missing. Build it now? ")
          (cljvindent-build-module)
          (cljvindent--load-module)))
      (error "Module cljvindent is not available")))

;;;###autoload
(defun cljvindent-install-module ()
  "Interactive entry point to build and load the module."
  (interactive)
  (cljvindent-build-module)
  (unless (cljvindent--load-module)
    (error "Module cljvindent:  built but could not be loaded"))
  (message "Native module cljvindent: is ready"))

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

(defun cljvindent--parent-form-data ()
  "Return plist data for the parent form at point, or nil."
  (save-excursion
    (let ((ppss (syntax-ppss)))
      (unless (or (nth 3 ppss) (nth 4 ppss))
        (condition-case nil
            (progn
              (unless (looking-at-p "\\s(")
                (backward-up-list 1))
              (let ((start (point))
                    (end (scan-sexps (point) 1)))
                (when end
                  (cljvindent--slice-form-data start end))))
          (error nil))))))

(defun cljvindent--immediate-parent-form-data ()
  "Return plist data for the immediate parent form at point, or nil."
  (save-excursion
    (let ((ppss (syntax-ppss)))
      (unless (or (nth 3 ppss) (nth 4 ppss))
        (condition-case nil
            (progn
              (unless (looking-at-p "\\s(")
                (backward-up-list 1))
              (let ((start (point))
                    (end (scan-sexps (point) 1)))
                (when end
                  (cljvindent--slice-form-data start end))))
          (error nil))))))

(defun cljvindent--outer-parent-form-data ()
  "Return plist data for the outer parent form at point, or nil."
  (save-excursion
    (let ((ppss (syntax-ppss)))
      (unless (or (nth 3 ppss) (nth 4 ppss))
        (condition-case nil
            (progn
              (unless (looking-at-p "\\s(")
                (backward-up-list 1))
              (backward-up-list 1)
              (let ((start (point))
                    (end (scan-sexps (point) 1)))
                (when end
                  (cljvindent--slice-form-data start end))))
          (error nil))))))

(defun cljvindent--region-form-data ()
  "Get the regions form data."
  (when (use-region-p)
    (cljvindent--slice-form-data (region-beginning) (region-end))))

(defun cljvindent--replace-form-with (form-data formatter-fn)
  "Will apply formatter result to the place where the form's bounds were.
Will use FORM-DATA(start, end etc), and the FORMATTER-FN on which will use from
the module."
  (unless form-data
    (error "No form/region found"))
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
(defun cljvindent-format-top-level-form ()
  "Will pick the parent form on where the cursor is and will try to indent it."
  (interactive)
  (unless (cljvindent--supported-mode-p)
    (error "Module cljvindent supports only clojure(script)/edn-mode"))
  (cljvindent--ensure-module)
  (let ((start (current-time)))
    (cljvindent--replace-form-with
     (cljvindent--top-level-form-data)
     #'cljvindent--indent-string)
    (message "cljvindent Done in: %.3fs"
             (float-time (time-subtract (current-time) start)))))

;;;###autoload
(defun cljvindent-format-parent ()
  "Will format the parent form of the form that the cursor currently is inside."
  (interactive)
  (unless (cljvindent--supported-mode-p)
    (error "Module cljvindent supports only clojure(script)/edn-mode"))
  (cljvindent--ensure-module)
  (let ((start (current-time)))
    (cljvindent--replace-form-with
     (cljvindent--parent-form-data)
     #'cljvindent--indent-string)
    (message "cljvindent Done in: %.3fs"
             (float-time (time-subtract (current-time) start)))))

;;;###autoload
(defun cljvindent-format-outer-parent ()
  "Will format the outer parent of the form that the cursor currenly is inside."
  (interactive)
  (unless (cljvindent--supported-mode-p)
    (error "Module cljvindent supports only clojure(script)/edn-mode"))
  (cljvindent--ensure-module)
  (let ((start (current-time)))
    (cljvindent--replace-form-with
     (cljvindent--outer-parent-form-data)
     #'cljvindent--indent-string)
    (message "cljvindent Done in: %.3fs"
             (float-time (time-subtract (current-time) start)))))

;;;###autoload
(defun cljvindent-format-region ()
  "Will extract the region and will try to indent it vertically."
  (interactive)
  (unless (cljvindent--supported-mode-p)
    (error "Module cljvindent supports only clojure(script)/edn-mode"))
  (cljvindent-ensure-module)
  (let ((start (current-time)))
    (cljvindent--replace-form-with
     (cljvindent--region-form-data)
     #'cljvindent--indent-string)
    (message "cljvindent Done in: %.3fs"
             (float-time (time-subtract (current-time) start)))))

;;;###autoload
(defun cljvindent-format-whole-buffer ()
  "Will vertically indent the whole file of the current buffer."
  (interactive)
  (unless (cljvindent--supported-mode-p)
    (error "Module cljvindent supports only clojure(script)/edn-mode"))
  (cljvindent--ensure-module)
  (let* ((start  (current-time))
         (result (cljvindent--indent-clj-file (buffer-file-name) cljvindent-enable-logs cljvindent-log-level cljvindent-log-file-output-type)))
    (revert-buffer :ignore-auto :noconfirm) ;; force buffer reload after full file update
    (message "cljvindent Done in: %.3fs"
             (float-time (time-subtract (current-time) start)))))

(provide 'cljvindent)

;;; cljvindent.el ends here
