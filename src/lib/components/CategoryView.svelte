<script lang="ts">
	import { createEventDispatcher } from 'svelte';
	import type { TreeNode } from '../stores/db';

	export let category: TreeNode | null = null;

	const dispatch = createEventDispatcher<{
		selectArticle: { id: number; slug: string };
		newArticle: void;
	}>();

	function handleArticleClick(article: { id: number; slug: string }) {
		dispatch('selectArticle', article);
	}

	function getAllArticles(node: TreeNode): { id: number; title: string; slug: string }[] {
		let articles = [...node.articles];
		for (const child of node.children) {
			articles = articles.concat(getAllArticles(child));
		}
		return articles;
	}

	function getIcon(icon: string | null): string {
		if (!icon) return '📁';
		if (icon.match(/^\p{Emoji}/u)) return icon;
		switch (icon) {
			case 'folder': return '📁';
			case 'code': return '💻';
			case 'book': return '📖';
			case 'globe': return '🌐';
			case 'star': return '⭐';
			default: return icon.length <= 2 ? icon : '📁';
		}
	}

	$: allArticles = category ? getAllArticles(category) : [];
</script>

{#if category}
	<div class="category-view">
		<div class="category-header">
			<div class="category-icon">{getIcon(category.icon)}</div>
			<h1 class="category-title">{category.name}</h1>
			<button class="btn btn-primary" on:click={() => dispatch('newArticle')}>
				+ New Article
			</button>
		</div>

		<div class="category-stats">
			<span>{allArticles.length} article{allArticles.length !== 1 ? 's' : ''}</span>
			{#if category.children.length > 0}
				<span>{category.children.length} subcategor{category.children.length !== 1 ? 'ies' : 'y'}</span>
			{/if}
		</div>

		{#if category.children.length > 0}
			<div class="category-section">
				<h2 class="section-title">Subcategories</h2>
				<div class="subcategories-grid">
					{#each category.children as child}
						<div class="subcategory-card">
							<span class="subcategory-icon">{getIcon(child.icon)}</span>
							<span class="subcategory-name">{child.name}</span>
							<span class="subcategory-count">{child.articles.length} articles</span>
						</div>
					{/each}
				</div>
			</div>
		{/if}

		{#if allArticles.length > 0}
			<div class="category-section">
				<h2 class="section-title">Articles</h2>
				<div class="articles-list">
					{#each allArticles as article}
						<div
							class="article-card"
							on:click={() => handleArticleClick(article)}
							on:keydown={(e) => e.key === 'Enter' && handleArticleClick(article)}
							role="button"
							tabindex="0"
						>
							<span class="article-icon">&#128196;</span>
							<span class="article-title">{article.title}</span>
						</div>
					{/each}
				</div>
			</div>
		{:else}
			<div class="empty-category">
				<div class="empty-icon">&#128194;</div>
				<p>No articles in this category yet.</p>
				<button class="btn btn-primary" on:click={() => dispatch('newArticle')}>
					Create First Article
				</button>
			</div>
		{/if}
	</div>
{/if}

<style>
	.category-view {
		max-width: 900px;
		margin: 0 auto;
	}

	.category-header {
		display: flex;
		align-items: center;
		gap: 16px;
		margin-bottom: 16px;
		padding-bottom: 16px;
		border-bottom: 1px solid var(--border-color);
	}

	.category-icon {
		font-size: 36px;
	}

	.category-title {
		flex: 1;
		font-size: 28px;
		font-weight: 600;
		margin: 0;
	}

	.category-stats {
		display: flex;
		gap: 16px;
		color: var(--text-secondary);
		font-size: 14px;
		margin-bottom: 24px;
	}

	.category-section {
		margin-bottom: 32px;
	}

	.section-title {
		font-size: 18px;
		font-weight: 600;
		margin-bottom: 16px;
		color: var(--text-secondary);
	}

	.subcategories-grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
		gap: 12px;
	}

	.subcategory-card {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 12px 16px;
		background-color: var(--bg-secondary);
		border: 1px solid var(--border-color);
		border-radius: 6px;
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.subcategory-card:hover {
		background-color: var(--bg-hover);
		border-color: var(--border-light);
	}

	.subcategory-icon {
		font-size: 20px;
	}

	.subcategory-name {
		flex: 1;
		font-weight: 500;
	}

	.subcategory-count {
		font-size: 12px;
		color: var(--text-muted);
	}

	.articles-list {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.article-card {
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 12px 16px;
		background-color: var(--bg-secondary);
		border: 1px solid var(--border-color);
		border-radius: 6px;
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.article-card:hover {
		background-color: var(--bg-hover);
		border-color: var(--accent);
	}

	.article-icon {
		font-size: 18px;
		color: var(--text-muted);
	}

	.article-title {
		font-weight: 500;
		color: var(--link-color);
	}

	.article-card:hover .article-title {
		color: var(--link-hover);
	}

	.empty-category {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		padding: 48px;
		text-align: center;
		color: var(--text-muted);
	}

	.empty-icon {
		font-size: 48px;
		margin-bottom: 16px;
		opacity: 0.5;
	}

	.empty-category p {
		margin-bottom: 16px;
	}
</style>
