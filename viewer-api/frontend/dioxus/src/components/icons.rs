//! SVG icon components ported from viewer-api TypeScript Icons.tsx.
//!
//! All icons accept optional `size`, `class`, and `color` props.

mod actions;
mod files;
mod navigation;
mod specialized;
mod status;

pub use self::{
    actions::{
        CloseIcon,
        FilterIcon,
        MinusIcon,
        PlusIcon,
        RefreshIcon,
        SearchIcon,
    },
    files::{
        DocumentIcon,
        FileIcon,
        FolderIcon,
        FolderOpenIcon,
    },
    navigation::{
        ChevronDownIcon,
        ChevronRightIcon,
    },
    specialized::{
        CodeIcon,
        CrateIcon,
        GraphIcon,
        HamburgerIcon,
        HomeIcon,
        LogIcon,
        ModuleIcon,
        SourceFileIcon,
        StatsIcon,
        ThemeIcon,
    },
    status::{
        AlertIcon,
        CheckIcon,
        InfoIcon,
    },
};
