;;; cljvindent-build.el --- Build and install the cljvindent native module -*- lexical-binding: t; -*-
;;; Commentary:
;; Build helpers for compiling and installing the cljvindent rust module.

(require 'cl-lib)
(require 'subr-x)

(defgroup cljvindent nil
  "Rust-backed package."
  :group 'applications)

(defcustom cljvindent-cargo-command "cargo"
  "Path to the cargo executable."
  :type 'string)

(defcustom cljvindent-enable-logs nil
  "Whether it should enable logs for each formatting."
  :group 'cljvindent
  :type 'boolean)

(defcustom cljvindent-log-level "info"
  "The level of logs that should show, default info."
  :group 'cljvindent
  :type '(choice
          (const :tag "Info" "info")
          (const :tag "Debug" "debug")))

(defcustom cljvindent-log-file-output-type "compact"
  "The type of logs that should save to file, default to compact."
  :group 'cljvindent
  :type '(choice
          (const :tag "Compact" "compact")
          (const :tag "JSON"    "json")))

(defcustom cljvindent-auto-build-module t
  "Whether `cljvindent' should offer to build the native module automatically."
  :type 'boolean)

(defun cljvindent--package-dir ()
  "Return the installed package directory."
  (file-name-directory
   (or load-file-name
       (locate-library "cljvindent-build"))))

(defun cljvindent--workspace-root ()
  "Return the package root."
  (let ((dir (cljvindent--package-dir)))
    (expand-file-name "." dir)))

(defun cljvindent--module-basename ()
  "Return the installed module basename, without extension."
  "clj_vindent_emacs_module")

(defun cljvindent--installed-module-file ()
  "Return the installed module path for current OS."
  (expand-file-name
   (concat (cljvindent--module-basename) module-file-suffix)
   (cljvindent--package-dir)))

(defun cljvindent--cargo-target-dir ()
  "Return the target/release directory."
  (expand-file-name "target/release/" (cljvindent--workspace-root)))

(defun cljvindent--built-module-candidates ()
  "Return possible module output filenames for the module."
  (let* ((base (cljvindent--module-basename))
         (suffix module-file-suffix)
         ;; cargo prefixes cdylib names with \"lib\" on Unix-like systems
         (plain (concat base suffix))
         (libprefixed (concat "lib" base suffix)))
    (delete-dups
     (list (expand-file-name plain (cljvindent--cargo-target-dir))
           (expand-file-name libprefixed (cljvindent--cargo-target-dir))))))

(defun cljvindent--find-built-module ()
  "Return the built module file path, or nil if not found."
  (seq-find #'file-exists-p (cljvindent--built-module-candidates)))


(defun cljvindent-build-module ()
  "Build the native module for cljvindent and install it in package dir."
  (interactive)
  (let* ((default-directory (cljvindent--workspace-root))
         (buf (get-buffer-create "*cljvindent-build*"))
         (status
          (process-file
           cljvindent-cargo-command
           nil buf t
           "build" "-p" "clj-vindent-emacs-module" "--release" "--lib")))
    (unless (eq status 0)
      (pop-to-buffer buf)
      (error "Module cljvindent: cargo build failed"))
    (let ((built (cljvindent--find-built-module))
          (dest (cljvindent--installed-module-file)))
      (unless built
        (pop-to-buffer buf)
        (error "Module cljvindent: built module not found in %s" (cljvindent--cargo-target-dir)))
      (copy-file built dest t)
      (message "Module cljvindent: installed native module to %s" dest)
      dest)))

;;;###autoload
(defun cljvindent-rebuild-module ()
  "Force a rebuild of the cljvindent native module."
  (interactive)
  (let ((dest (cljvindent--installed-module-file)))
    (when (file-exists-p dest)
      (delete-file dest))
    (cljvindent-build-module)))

(provide 'cljvindent-build)

;;; cljvindent-build.el ends here
