<script lang="ts">
	import { createEventDispatcher } from 'svelte';
	import type { SearchResult } from '../stores/db';

	export let results: SearchResult[] = [];
	export let isLoading = false;

	const dispatch = createEventDispatcher<{
		search: { query: string };
		select: { id: number; slug: string };
		close: void;
	}>();

	let query = '';
	let showResults = false;
	let selectedIndex = 0;
	let inputElement: HTMLInputElement;

	$: if (query.length >= 2) {
		dispatch('search', { query });
		showResults = true;
		selectedIndex = 0;
	} else {
		showResults = false;
	}

	function handleKeydown(event: KeyboardEvent) {
		if (!showResults || results.length === 0) return;

		switch (event.key) {
			case 'ArrowDown':
				event.preventDefault();
				selectedIndex = Math.min(selectedIndex + 1, results.length - 1);
				break;
			case 'ArrowUp':
				event.preventDefault();
				selectedIndex = Math.max(selectedIndex - 1, 0);
				break;
			case 'Enter':
				event.preventDefault();
				if (results[selectedIndex]) {
					selectResult(results[selectedIndex]);
				}
				break;
			case 'Escape':
				event.preventDefault();
				close();
				break;
		}
	}

	function selectResult(result: SearchResult) {
		dispatch('select', { id: result.id, slug: result.slug });
		close();
	}

	function close() {
		query = '';
		showResults = false;
		dispatch('close');
	}

	export function focus() {
		inputElement?.focus();
	}

	function stripHtml(html: string): string {
		return html.replace(/<[^>]*>/g, '');
	}
</script>

<div class="search-container">
	<span class="search-icon">&#128269;</span>
	<input
		type="text"
		class="search-input"
		placeholder="Search articles... (Ctrl+K)"
		bind:value={query}
		bind:this={inputElement}
		on:keydown={handleKeydown}
		on:focus={() => query.length >= 2 && (showResults = true)}
		on:blur={() => setTimeout(() => (showResults = false), 200)}
	/>

	{#if showResults}
		<div class="search-results">
			{#if isLoading}
				<div class="search-loading">
					<div class="spinner"></div>
					Searching...
				</div>
			{:else if results.length === 0}
				<div class="search-no-results">
					No results found for "{query}"
				</div>
			{:else}
				{#each results as result, index (result.id)}
					<div
						class="search-result"
						class:selected={index === selectedIndex}
						on:click={() => selectResult(result)}
						on:mouseenter={() => (selectedIndex = index)}
						role="button"
						tabindex="0"
						on:keydown={(e) => e.key === 'Enter' && selectResult(result)}
					>
						<div class="search-result-title">{result.title}</div>
						<div class="search-result-snippet">{@html result.snippet}</div>
					</div>
				{/each}
			{/if}
		</div>
	{/if}
</div>

<style>
	.search-container {
		position: relative;
		flex: 1;
		max-width: 500px;
	}

	.search-input {
		width: 100%;
		padding: 8px 16px 8px 36px;
		background-color: var(--bg-tertiary);
		border: 1px solid var(--border-color);
		border-radius: 4px;
		color: var(--text-primary);
		font-size: 14px;
	}

	.search-input:focus {
		outline: none;
		border-color: var(--accent);
	}

	.search-input::placeholder {
		color: var(--text-muted);
	}

	.search-icon {
		position: absolute;
		left: 10px;
		top: 50%;
		transform: translateY(-50%);
		color: var(--text-muted);
		font-size: 14px;
		pointer-events: none;
	}

	.search-results {
		position: absolute;
		top: 100%;
		left: 0;
		right: 0;
		margin-top: 4px;
		background-color: var(--bg-secondary);
		border: 1px solid var(--border-color);
		border-radius: 4px;
		max-height: 400px;
		overflow-y: auto;
		z-index: 100;
		box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
	}

	.search-result {
		padding: 10px 16px;
		cursor: pointer;
		border-bottom: 1px solid var(--border-color);
	}

	.search-result:last-child {
		border-bottom: none;
	}

	.search-result:hover,
	.search-result.selected {
		background-color: var(--bg-hover);
	}

	.search-result-title {
		font-weight: 500;
		margin-bottom: 4px;
		color: var(--text-primary);
	}

	.search-result-snippet {
		font-size: 13px;
		color: var(--text-secondary);
		line-height: 1.4;
	}

	.search-result-snippet :global(mark) {
		background-color: rgba(74, 158, 255, 0.3);
		color: inherit;
		padding: 0 2px;
		border-radius: 2px;
	}

	.search-loading,
	.search-no-results {
		padding: 16px;
		text-align: center;
		color: var(--text-secondary);
	}

	.search-loading {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 8px;
	}

	.spinner {
		width: 16px;
		height: 16px;
		border: 2px solid var(--border-color);
		border-top-color: var(--accent);
		border-radius: 50%;
		animation: spin 0.8s linear infinite;
	}

	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
	}
</style>
