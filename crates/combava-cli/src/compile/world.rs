//! Le `World` Typst et son système de fichiers virtuel (spécification,
//! section 8).
//!
//! - `paths::MAIN` : le code généré, en mémoire
//! - `paths::TEMPLATE_DIR/…` : le dossier du template, sur le disque ou embarqué
//! - `paths::BIBLIOGRAPHY` : le fichier `.bib`
//! - `/__combava__/…` : réservé, introuvable
//! - tout autre `/x` : `<dossier du .md>/x`, en lecture seule

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use combava_core::paths;
use typst::diag::{FileError, FileResult, PackageError};
use typst::foundations::{Bytes, Datetime, Duration};
use typst::syntax::package::PackageSpec;
use typst::syntax::{FileId, RootedPath, Source, VirtualPath, VirtualRoot};
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::{Library, LibraryExt, World};
use typst_kit::datetime::Time;
use typst_kit::downloader::SystemDownloader;
use typst_kit::files::{FileLoader, FileStore, FsRoot};
use typst_kit::fonts::{self, FontStore};
use typst_kit::packages::SystemPackages;

use crate::template::{Fonts, Template};

/// Préfixe des chemins réservés à Combava.
const RESERVED: &str = "/__combava__";

/// Ce que le `World` sert, en plus des polices.
pub struct Files {
    /// Le code Typst généré par le core.
    pub main: String,
    /// Dossier du fichier markdown, racine des chemins `/x`.
    pub root: PathBuf,
    pub template: Template,
    pub bibliography: Option<PathBuf>,
}

pub struct CombavaWorld {
    library: LazyHash<Library>,
    fonts: FontStore,
    files: FileStore<Loader>,
    main: FileId,
    time: Time,
}

impl CombavaWorld {
    pub fn new(files: Files) -> Self {
        let mut fonts = FontStore::new();
        match files.template.fonts() {
            Fonts::Dir(Some(dir)) => fonts.extend(fonts::scan(&dir)),
            Fonts::Dir(None) => {}
            Fonts::Embedded(data) => {
                for data in data {
                    fonts.extend(Font::iter(Bytes::new(data)).map(|font| {
                        let info = font.info().clone();
                        (font, info)
                    }));
                }
            }
        }
        fonts.extend(fonts::system());
        fonts.extend(fonts::embedded());

        let user_agent = concat!("combava/", env!("CARGO_PKG_VERSION"));
        Self {
            library: LazyHash::new(Library::default()),
            fonts,
            files: FileStore::new(Loader {
                main: Bytes::from_string(files.main),
                root: files.root,
                template: files.template,
                bibliography: files.bibliography,
                packages: SystemPackages::new(SystemDownloader::new(user_agent)),
                download_failure: Mutex::new(None),
            }),
            main: id(paths::MAIN),
            time: Time::system(),
        }
    }

    pub fn main_id(&self) -> FileId {
        self.main
    }

    /// Le premier package dont le téléchargement a échoué, et pourquoi.
    pub fn download_failure(&self) -> Option<(PackageSpec, String)> {
        self.files.loader().download_failure.lock().ok()?.clone()
    }

    /// Chemin affiché pour un fichier du `World` dans les diagnostics.
    pub fn display_path(&self, id: FileId) -> PathBuf {
        self.files.loader().display_path(id)
    }
}

impl World for CombavaWorld {
    fn library(&self) -> &LazyHash<Library> {
        &self.library
    }

    fn book(&self) -> &LazyHash<FontBook> {
        self.fonts.book()
    }

    fn main(&self) -> FileId {
        self.main
    }

    fn source(&self, id: FileId) -> FileResult<Source> {
        self.files.source(id)
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        self.files.file(id)
    }

    fn font(&self, index: usize) -> Option<Font> {
        self.fonts.font(index)
    }

    fn today(&self, offset: Option<Duration>) -> Option<Datetime> {
        self.time.today(offset)
    }
}

/// L'identifiant d'un chemin du projet.
fn id(path: &str) -> FileId {
    let path = VirtualPath::new(path).expect("chemin virtuel valide");
    RootedPath::new(VirtualRoot::Project, path).intern()
}

struct Loader {
    main: Bytes,
    root: PathBuf,
    template: Template,
    bibliography: Option<PathBuf>,
    packages: SystemPackages,
    download_failure: Mutex<Option<(PackageSpec, String)>>,
}

impl Loader {
    /// Le fichier du template désigné par un chemin virtuel, sans `/` de tête.
    fn template_file(path: &str) -> Option<&str> {
        path.strip_prefix(paths::TEMPLATE_DIR)?.strip_prefix('/')
    }

    fn display_path(&self, id: FileId) -> PathBuf {
        let vpath = id.vpath();
        if let VirtualRoot::Package(spec) = id.root() {
            return PathBuf::from(format!("{spec}{}", vpath.get_with_slash()));
        }
        let path = vpath.get_with_slash();
        if let Some(relative) = Self::template_file(path) {
            self.template.display_path(relative)
        } else if path == paths::BIBLIOGRAPHY
            && let Some(bibliography) = &self.bibliography
        {
            bibliography.clone()
        } else {
            vpath
                .realize(&self.root)
                .unwrap_or_else(|_| PathBuf::from(path))
        }
    }

    fn package(&self, spec: &PackageSpec, vpath: &VirtualPath) -> FileResult<Bytes> {
        match self.packages.obtain(spec) {
            Ok(root) => root.load(vpath),
            Err(error) => {
                if let PackageError::NetworkFailed(reason) = &error
                    && let Ok(mut failure) = self.download_failure.lock()
                {
                    let reason = reason.as_deref().unwrap_or("raison inconnue").to_string();
                    failure.get_or_insert((spec.clone(), reason));
                }
                Err(FileError::Package(error))
            }
        }
    }
}

impl FileLoader for Loader {
    fn load(&self, id: FileId) -> FileResult<Bytes> {
        let vpath = id.vpath();
        if let VirtualRoot::Package(spec) = id.root() {
            return self.package(spec, vpath);
        }
        let path = vpath.get_with_slash();
        if path == paths::MAIN {
            return Ok(self.main.clone());
        }
        if path == paths::BIBLIOGRAPHY
            && let Some(bibliography) = &self.bibliography
        {
            return read(bibliography);
        }
        if let Some(relative) = Self::template_file(path) {
            return match self.template.read(relative) {
                Some(Ok(data)) => Ok(Bytes::new(data)),
                Some(Err(error)) => Err(FileError::from_io(error, &self.display_path(id))),
                None => Err(FileError::NotFound(self.display_path(id))),
            };
        }
        if path == RESERVED || path.starts_with(&format!("{RESERVED}/")) {
            return Err(FileError::NotFound(path.into()));
        }
        FsRoot::new(self.root.clone()).load(vpath)
    }
}

fn read(path: &Path) -> FileResult<Bytes> {
    std::fs::read(path)
        .map(Bytes::new)
        .map_err(|error| FileError::from_io(error, path))
}
