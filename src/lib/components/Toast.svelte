<script lang="ts">
	import { toasts } from '../stores/toast';
	import { fly, fade } from 'svelte/transition';

	const icons = {
		success: '✓',
		error: '✕',
		warning: '⚠',
		info: 'ℹ'
	};
</script>

<div class="toast-container">
	{#each $toasts as toast (toast.id)}
		<div
			class="toast toast-{toast.type}"
			transition:fly={{ y: 50, duration: 300 }}
		>
			<span class="toast-icon">{icons[toast.type]}</span>
			<span class="toast-message">{toast.message}</span>
			<button class="toast-close" on:click={() => toasts.remove(toast.id)}>×</button>
		</div>
	{/each}
</div>

<style>
	.toast-container {
		position: fixed;
		bottom: 24px;
		right: 24px;
		display: flex;
		flex-direction: column;
		gap: 8px;
		z-index: 9999;
		pointer-events: none;
	}

	.toast {
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 12px 16px;
		background-color: var(--bg-secondary);
		border: 1px solid var(--border-color);
		border-radius: 8px;
		box-shadow: 0 4px 20px rgba(0, 0, 0, 0.4);
		min-width: 280px;
		max-width: 400px;
		pointer-events: auto;
	}

	.toast-icon {
		width: 24px;
		height: 24px;
		border-radius: 50%;
		display: flex;
		align-items: center;
		justify-content: center;
		font-size: 14px;
		font-weight: bold;
		flex-shrink: 0;
	}

	.toast-success .toast-icon {
		background-color: rgba(34, 197, 94, 0.2);
		color: #22c55e;
	}

	.toast-error .toast-icon {
		background-color: rgba(239, 68, 68, 0.2);
		color: #ef4444;
	}

	.toast-warning .toast-icon {
		background-color: rgba(245, 158, 11, 0.2);
		color: #f59e0b;
	}

	.toast-info .toast-icon {
		background-color: rgba(124, 92, 191, 0.2);
		color: var(--accent);
	}

	.toast-message {
		flex: 1;
		font-size: 14px;
		color: var(--text-primary);
	}

	.toast-close {
		background: none;
		border: none;
		color: var(--text-muted);
		font-size: 20px;
		cursor: pointer;
		padding: 0;
		line-height: 1;
		opacity: 0.6;
		transition: opacity 0.15s;
	}

	.toast-close:hover {
		opacity: 1;
	}
</style>
