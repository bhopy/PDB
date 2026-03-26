<script lang="ts">
	import { createEventDispatcher, onMount } from 'svelte';
	import { fade, scale } from 'svelte/transition';
	import type { TreeNode } from '../stores/db';

	export let show = false;
	export let tree: TreeNode[] = [];

	const dispatch = createEventDispatcher<{
		select: { id: number; slug: string; type: 'article' | 'category' };
		close: void;
	}>();

	let query = '';
	let inputEl: HTMLInputElement;
	let selectedIndex = 0;

	interface SearchItem {
		id: number;
		title: string;
		slug: string;
		type: 'article' | 'category';
		path: string;
		icon: string;
	}

	// Flatten tree into searchable items
	function getAllItems(nodes: TreeNode[], path = ''): SearchItem[] {
		let items: SearchItem[] = [];
		for (const node of nodes) {
			const nodePath = path ? `${path} / ${node.name}` : node.name;

			// Add category
			items.push({
				id: node.id,
				title: node.name,
				slug: '',
				type: 'category',
				path: path || 'Root',
				icon: '📁'
			});

			// Add articles
			for (const article of node.articles) {
				items.push({
					id: article.id,
					title: article.title,
					slug: article.slug,
					type: 'article',
					path: nodePath,
					icon: '📄'
				});
			}

			// Recurse into children
			items = items.concat(getAllItems(node.children, nodePath));
		}
		return items;
	}

	// Simple fuzzy match
	function fuzzyMatch(text: string, pattern: string): { match: boolean; score: number } {
		const lowerText = text.toLowerCase();
		const lowerPattern = pattern.toLowerCase();

		if (lowerPattern.length === 0) return { match: true, score: 0 };
		if (lowerText.includes(lowerPattern)) return { match: true, score: 100 - lowerText.indexOf(lowerPattern) };

		let patternIdx = 0;
		let score = 0;
		let lastMatchIdx = -1;

		for (let i = 0; i < lowerText.length && patternIdx < lowerPattern.length; i++) {
			if (lowerText[i] === lowerPattern[patternIdx]) {
				score += lastMatchIdx === i - 1 ? 10 : 5; // Consecutive matches score higher
				lastMatchIdx = i;
				patternIdx++;
			}
		}

		return {
			match: patternIdx === lowerPattern.length,
			score
		};
	}

	$: allItems = getAllItems(tree);
	$: filteredItems = query
		? allItems
			.map(item => ({ ...item, ...fuzzyMatch(item.title, query) }))
			.filter(item => item.match)
			.sort((a, b) => b.score - a.score)
			.slice(0, 10)
		: allItems.slice(0, 10);

	$: if (filteredItems.length > 0 && selectedIndex >= filteredItems.length) {
		selectedIndex = filteredItems.length - 1;
	}

	$: if (show && inputEl) {
		setTimeout(() => inputEl?.focus(), 50);
	}

	function handleKeydown(e: KeyboardEvent) {
		switch (e.key) {
			case 'ArrowDown':
				e.preventDefault();
				selectedIndex = Math.min(selectedIndex + 1, filteredItems.length - 1);
				break;
			case 'ArrowUp':
				e.preventDefault();
				selectedIndex = Math.max(selectedIndex - 1, 0);
				break;
			case 'Enter':
				e.preventDefault();
				if (filteredItems[selectedIndex]) {
					selectItem(filteredItems[selectedIndex]);
				}
				break;
			case 'Escape':
				e.preventDefault();
				close();
				break;
		}
	}

	function selectItem(item: SearchItem) {
		dispatch('select', { id: item.id, slug: item.slug, type: item.type });
		close();
	}

	function close() {
		query = '';
		selectedIndex = 0;
		dispatch('close');
	}
</script>

{#if show}
	<div class="quick-switcher-overlay" on:click={close} transition:fade={{ duration: 150 }}>
		<div class="quick-switcher" on:click|stopPropagation transition:scale={{ duration: 150, start: 0.95 }}>
			<div class="quick-switcher-input-wrapper">
				<span class="search-icon">🔍</span>
				<input
					bind:this={inputEl}
					bind:value={query}
					on:keydown={handleKeydown}
					type="text"
					placeholder="Search articles and categories..."
					class="quick-switcher-input"
				/>
				<kbd class="kbd">esc</kbd>
			</div>

			<div class="quick-switcher-results">
				{#if filteredItems.length === 0}
					<div class="no-results">No results found</div>
				{:else}
					{#each filteredItems as item, i}
						<div
							class="result-item"
							class:selected={i === selectedIndex}
							on:click={() => selectItem(item)}
							on:mouseenter={() => selectedIndex = i}
							role="option"
							aria-selected={i === selectedIndex}
						>
							<span class="result-icon">{item.icon}</span>
							<div class="result-content">
								<span class="result-title">{item.title}</span>
								<span class="result-path">{item.path}</span>
							</div>
							<span class="result-type">{item.type}</span>
						</div>
					{/each}
				{/if}
			</div>

			<div class="quick-switcher-footer">
				<span><kbd>↑↓</kbd> navigate</span>
				<span><kbd>↵</kbd> select</span>
				<span><kbd>esc</kbd> close</span>
			</div>
		</div>
	</div>
{/if}

<style>
	.quick-switcher-overlay {
		position: fixed;
		inset: 0;
		background-color: rgba(0, 0, 0, 0.7);
		display: flex;
		align-items: flex-start;
		justify-content: center;
		padding-top: 15vh;
		z-index: 10000;
	}

	.quick-switcher {
		width: 560px;
		max-width: 90vw;
		background-color: var(--bg-secondary);
		border: 1px solid var(--border-color);
		border-radius: 12px;
		box-shadow: 0 20px 60px rgba(0, 0, 0, 0.5);
		overflow: hidden;
	}

	.quick-switcher-input-wrapper {
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 16px;
		border-bottom: 1px solid var(--border-color);
	}

	.search-icon {
		font-size: 18px;
		opacity: 0.6;
	}

	.quick-switcher-input {
		flex: 1;
		background: none;
		border: none;
		color: var(--text-primary);
		font-size: 16px;
		outline: none;
	}

	.quick-switcher-input::placeholder {
		color: var(--text-muted);
	}

	.kbd {
		background-color: var(--bg-tertiary);
		border: 1px solid var(--border-color);
		border-radius: 4px;
		padding: 2px 6px;
		font-size: 11px;
		color: var(--text-muted);
		font-family: inherit;
	}

	.quick-switcher-results {
		max-height: 400px;
		overflow-y: auto;
	}

	.no-results {
		padding: 24px;
		text-align: center;
		color: var(--text-muted);
	}

	.result-item {
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 10px 16px;
		cursor: pointer;
		transition: background-color 0.1s;
	}

	.result-item:hover,
	.result-item.selected {
		background-color: var(--bg-hover);
	}

	.result-item.selected {
		background-color: var(--accent);
		background-color: rgba(124, 92, 191, 0.3);
	}

	.result-icon {
		font-size: 18px;
		width: 24px;
		text-align: center;
	}

	.result-content {
		flex: 1;
		min-width: 0;
	}

	.result-title {
		display: block;
		font-weight: 500;
		color: var(--text-primary);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.result-path {
		display: block;
		font-size: 12px;
		color: var(--text-muted);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.result-type {
		font-size: 11px;
		color: var(--text-muted);
		background-color: var(--bg-tertiary);
		padding: 2px 8px;
		border-radius: 4px;
		text-transform: uppercase;
	}

	.quick-switcher-footer {
		display: flex;
		gap: 16px;
		padding: 10px 16px;
		border-top: 1px solid var(--border-color);
		font-size: 12px;
		color: var(--text-muted);
	}

	.quick-switcher-footer kbd {
		margin-right: 4px;
	}
</style>
