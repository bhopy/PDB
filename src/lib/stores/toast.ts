import { writable } from 'svelte/store';

export interface Toast {
	id: number;
	message: string;
	type: 'success' | 'error' | 'info' | 'warning';
	duration: number;
}

function createToastStore() {
	const { subscribe, update } = writable<Toast[]>([]);
	let nextId = 0;

	function add(message: string, type: Toast['type'] = 'info', duration = 3000) {
		const id = nextId++;
		const toast: Toast = { id, message, type, duration };

		update(toasts => [...toasts, toast]);

		if (duration > 0) {
			setTimeout(() => {
				remove(id);
			}, duration);
		}

		return id;
	}

	function remove(id: number) {
		update(toasts => toasts.filter(t => t.id !== id));
	}

	return {
		subscribe,
		success: (message: string, duration?: number) => add(message, 'success', duration),
		error: (message: string, duration?: number) => add(message, 'error', duration),
		info: (message: string, duration?: number) => add(message, 'info', duration),
		warning: (message: string, duration?: number) => add(message, 'warning', duration),
		remove
	};
}

export const toasts = createToastStore();
