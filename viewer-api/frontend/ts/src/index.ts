// Re-export preact and signals to ensure single instance across consuming packages
export * from 'preact';
export * from 'preact/hooks';
export * as signals from '@preact/signals';
export { signal, computed, effect, batch } from '@preact/signals';
export type { Signal, ReadonlySignal } from '@preact/signals';

// Re-export common components
export * from './components/TreeView';
export * from './components/Spinner';
export * from './components/TabBar';
export * from './components/Icons';
export * from './components/Header';
export * from './components/Sidebar';
export * from './components/ResizeHandle';
export * from './components/Layout';
export * from './components/Panel';
export * from './components/CodeViewer';
export * from './components/FileContentViewer';
export * from './components/FileTree';

// Re-export session utilities
export * from './session';

// Re-export URL state management
export * from './url-state';

// Re-export shared theme system
export * from './store/theme';

// Re-export GPU overlay system
export * from './components/WgpuOverlay/WgpuOverlay';
export * from './components/WgpuOverlay/schemas';
export * from './components/WgpuOverlay/element-types';
export * from './components/Scene3D/Scene3D';
export * from './utils/math3d';
export * from './effects/palette';

// Re-export Graph3DView component and public types
export { Graph3DView } from './components/Graph3DView/Graph3DView';
export type { Graph3DNode, Graph3DEdge, Graph3DViewProps } from './components/Graph3DView/types';

// Re-export HypergraphViewCore component and public types
export { HypergraphViewCore } from './components/HypergraphView/HypergraphViewCore';
export type { HypergraphViewProps, NestingSettings } from './components/HypergraphView/types';
export type { GraphLayout, LayoutNode, LayoutEdge } from './components/HypergraphView/layout';

// Re-export shared ThemeSettings component and store types
export { ThemeSettings } from './components/ThemeSettings/ThemeSettings';
export type { ThemeSettingsStore, SavedTheme, ColorTheme, EffectTheme } from './store/theme';
export { DEFAULT_EFFECT_SETTINGS_OFF } from './store/theme';
