import { writable } from 'svelte/store';

// Shared drag state across all TreeNav instances
export const globalDraggedItem = writable<{
	type: 'category' | 'article';
	id: number;
	sourceCategory?: number;
} | null>(null);

export const globalDropTarget = writable<{
	id: number;
	position: 'before' | 'after' | 'inside';
	targetCategory: number;
} | null>(null);
