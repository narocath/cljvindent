;;; cljvindent-build.el --- Build and install the cljvindent native module -*- lexical-binding: t; -*-

;; Author: Panagiotis Koromilias
;; Version: 0.1.0

;; URL: https://github.com/narocath/cljvindent-emacs

;;; Commentary:
;; Build helpers for compiling and installing the cljvindent native module.

;;; Code:

(require 'cl-lib)

(defconst cljvindent--module-name "cljvindent"
  "Base name of the cljvindent native module.")

(defgroup cljvindent nil
  "Clojure, Clojurescript and EDN, indentation with a native module."
  :group 'applications)

(defcustom cljvindent-build-command
  '("cargo" "build" "--release" "--features" "emacs-module")
  "Full command used to build the cljvindent native module."
  :group 'cljvindent
  :type '(repeat string))

(defcustom cljvindent-enable-logs nil
  "Whether to enable logs for each indentation call."
  :group 'cljvindent
  :type 'boolean)

(defcustom cljvindent-log-level "info"
  "The log level to use."
  :group 'cljvindent
  :type '(choice
          (const :tag "Info" "info")
          (const :tag "Debug" "debug")))

(defcustom cljvindent-log-file-output-type "compact"
  "The log file output format."
  :group 'cljvindent
  :type '(choice
          (const :tag "Compact" "compact")
          (const :tag "JSON" "json")))

(defcustom cljvindent-auto-build-module t
  "Whether `cljvindent' should offer to build the native module automatically."
  :group 'cljvindent
  :type 'boolean)

(defconst cljvindent--package-dir
  (file-name-directory
   (or load-file-name
       (locate-library "cljvindent-build")))
  "Directory where cljvindent is installed.")

(defun cljvindent--package-dir ()
  "Return the installed package directory."
  cljvindent--package-dir)

(defun cljvindent--project-root ()
  "Return the cljvindent project root."
  cljvindent--package-dir)

(defun cljvindent--rust-project-dir ()
  "Return the native's module project directory."
  (expand-file-name "clj-vindent-engine" cljvindent--package-dir))

(defun cljvindent--cargo-manifest-file ()
  "Return the Cargo.toml path for the native project."
  (expand-file-name "Cargo.toml"
                    (cljvindent--rust-project-dir)))

(defun cljvindent--installed-module-file ()
  "Return the installed module path."
  (expand-file-name
   (concat cljvindent--module-name module-file-suffix)
   (cljvindent--package-dir)))

(defun cljvindent--cargo-target-dir ()
  "Return the module target/release directory."
  (expand-file-name "target/release/"
                    (cljvindent--rust-project-dir)))

(defun cljvindent--built-module-candidates ()
  "Return possible built module filenames."
  (let* ((suffix module-file-suffix)
         (plain (concat cljvindent--module-name suffix))
         (libprefixed (concat "lib" cljvindent--module-name suffix)))
    (delete-dups
     (list (expand-file-name plain (cljvindent--cargo-target-dir))
           (expand-file-name libprefixed (cljvindent--cargo-target-dir))))))

(defun cljvindent--find-built-module ()
  "Return the built module file path, or nil if not found."
  (cl-find-if #'file-exists-p (cljvindent--built-module-candidates)))

(defun cljvindent--find-executable (program)
  "Return PROGRAM if it is executable, otherwise nil."
  (or (executable-find program)
      (let ((expanded (expand-file-name program)))
        (when (file-executable-p expanded)
          expanded))))

(defun cljvindent--ensure-rust-toolchain ()
  "Ensure the configured build tool and rustc are available."
  (let ((program (car cljvindent-build-command)))
    (unless program
      (user-error "`cljvindent-build-command' is empty"))
    (unless (cljvindent--find-executable program)
      (user-error "Could not find build program: %s" program)))
  (unless (cljvindent--find-executable "rustc")
    (user-error "Could not find rustc executable")))

(defun cljvindent-build-module ()
  "Build the cljvindent native module and install it in the package directory."
  (interactive)
  (cljvindent--ensure-rust-toolchain)
  (let* ((rust-dir (cljvindent--rust-project-dir))
         (manifest (cljvindent--cargo-manifest-file))
         (command cljvindent-build-command)
         (program (car command))
         (args (cdr command))
         (buf (get-buffer-create "*cljvindent-build*"))
         (needs-restart (bound-and-true-p cljvindent--module-loaded)))
    (unless (file-directory-p rust-dir)
      (user-error "Module's project directory does not exist: %s" rust-dir))
    (unless (file-exists-p manifest)
      (user-error "No Cargo.toml found in: %s" manifest))
    (let ((default-directory rust-dir)
          (status (apply #'process-file program nil buf t args)))
      (unless (eq status 0)
        (pop-to-buffer buf)
        (user-error "Module cljvindent: build failed")))
    (let ((built (cljvindent--find-built-module))
          (dest (cljvindent--installed-module-file)))
      (unless built
        (pop-to-buffer buf)
        (user-error "Module cljvindent: built module not found in %s"
                    (cljvindent--cargo-target-dir)))
      (copy-file built dest t)
      (message
       (if needs-restart
           "Module cljvindent: rebuilt at %s. Restart Emacs to use the new version"
         "Module cljvindent: installed native module to %s")
       dest)
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
