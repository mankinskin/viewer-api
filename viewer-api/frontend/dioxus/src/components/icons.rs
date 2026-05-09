//! SVG icon components ported from viewer-api TypeScript Icons.tsx.
//!
//! All icons accept optional `size`, `class`, and `color` props.

mod actions;
mod files;
mod navigation;
mod specialized;
mod status;

pub use self::actions::{CloseIcon, FilterIcon, MinusIcon, PlusIcon, RefreshIcon, SearchIcon};
pub use self::files::{DocumentIcon, FileIcon, FolderIcon, FolderOpenIcon};
pub use self::navigation::{ChevronDownIcon, ChevronRightIcon};
pub use self::specialized::{
    CodeIcon, CrateIcon, GraphIcon, HamburgerIcon, HomeIcon, LogIcon, ModuleIcon,
    SourceFileIcon, StatsIcon,
};
pub use self::status::{AlertIcon, CheckIcon, InfoIcon};
