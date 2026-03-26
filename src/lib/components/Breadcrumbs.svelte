<script lang="ts">
	import { createEventDispatcher } from 'svelte';
	import type { TreeNode, ArticleWithTags } from '../stores/db';

	export let tree: TreeNode[] = [];
	export let currentArticle: ArticleWithTags | null = null;
	export let currentCategoryId: number | null = null;

	const dispatch = createEventDispatcher<{
		selectCategory: { id: number };
	}>();

	interface BreadcrumbItem {
		id: number;
		name: string;
		type: 'category';
	}

	function findPath(nodes: TreeNode[], targetId: number, targetType: 'category' | 'article', path: BreadcrumbItem[] = []): BreadcrumbItem[] | null {
		for (const node of nodes) {
			const newPath = [...path, { id: node.id, name: node.name, type: 'category' as const }];

			if (targetType === 'category' && node.id === targetId) {
				return newPath;
			}

			if (targetType === 'article') {
				const article = node.articles.find(a => a.id === targetId);
				if (article) {
					return newPath;
				}
			}

			const found = findPath(node.children, targetId, targetType, newPath);
			if (found) return found;
		}
		return null;
	}

	$: breadcrumbs = (() => {
		if (currentArticle) {
			return findPath(tree, currentArticle.article.id, 'article') || [];
		} else if (currentCategoryId) {
			return findPath(tree, currentCategoryId, 'category') || [];
		}
		return [];
	})();

	function handleClick(item: BreadcrumbItem) {
		dispatch('selectCategory', { id: item.id });
	}
</script>

{#if breadcrumbs.length > 0}
	<nav class="breadcrumbs" aria-label="Breadcrumb">
		{#each breadcrumbs as item, i}
			{#if i > 0}
				<span class="breadcrumb-separator">/</span>
			{/if}
			<button
				class="breadcrumb-item"
				class:current={i === breadcrumbs.length - 1 && !currentArticle}
				on:click={() => handleClick(item)}
			>
				{item.name}
			</button>
		{/each}
		{#if currentArticle}
			<span class="breadcrumb-separator">/</span>
			<span class="breadcrumb-item current">{currentArticle.article.title}</span>
		{/if}
	</nav>
{/if}

<style>
	.breadcrumbs {
		display: flex;
		align-items: center;
		flex-wrap: wrap;
		gap: 4px;
		font-size: 13px;
		padding: 8px 0;
		margin-bottom: 8px;
	}

	.breadcrumb-separator {
		color: var(--text-muted);
		margin: 0 4px;
	}

	.breadcrumb-item {
		background: none;
		border: none;
		color: var(--text-secondary);
		cursor: pointer;
		padding: 4px 8px;
		border-radius: 4px;
		transition: all 0.15s;
	}

	.breadcrumb-item:hover:not(.current) {
		background-color: var(--bg-hover);
		color: var(--link-color);
	}

	.breadcrumb-item.current {
		color: var(--text-primary);
		cursor: default;
		font-weight: 500;
	}
</style>
