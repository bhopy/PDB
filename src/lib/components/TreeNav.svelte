<script lang="ts">
	import { createEventDispatcher } from 'svelte';
	import { flip } from 'svelte/animate';
	import { fade, slide } from 'svelte/transition';
	import type { TreeNode } from '../stores/db';
	import { globalDraggedItem, globalDropTarget } from '../stores/dragState';

	export let nodes: TreeNode[] = [];
	export let selectedArticleId: number | null = null;
	export let selectedCategoryId: number | null = null;
	export let depth = 0;

	const dispatch = createEventDispatcher<{
		selectArticle: { id: number; slug: string };
		selectCategory: { id: number };
		contextMenu: { event: MouseEvent; type: 'category' | 'article'; id: number; parentId?: number };
		createArticle: { categoryId: number };
		moveItem: { type: 'category' | 'article'; id: number; targetCategoryId: number | null; position: number };
	}>();

	let expandedNodes = new Set<number>([1]); // Root expanded by default

	function toggleNode(id: number) {
		if (expandedNodes.has(id)) {
			expandedNodes.delete(id);
		} else {
			expandedNodes.add(id);
		}
		expandedNodes = expandedNodes;
	}

	function handleArticleClick(article: { id: number; slug: string }) {
		dispatch('selectArticle', article);
	}

	function handleCategoryClick(id: number) {
		dispatch('selectCategory', { id });
	}

	function handleContextMenu(event: MouseEvent, type: 'category' | 'article', id: number, parentId?: number) {
		event.preventDefault();
		dispatch('contextMenu', { event, type, id, parentId });
	}

	function handleDoubleClick(categoryId: number) {
		dispatch('createArticle', { categoryId });
	}

	// Drag and drop handlers
	function handleDragStart(event: DragEvent, type: 'category' | 'article', id: number, sourceCategory?: number) {
		globalDraggedItem.set({ type, id, sourceCategory });
		if (event.dataTransfer) {
			event.dataTransfer.effectAllowed = 'move';
			event.dataTransfer.setData('text/plain', JSON.stringify({ type, id, sourceCategory }));
		}
		document.body.classList.add('dragging');
	}

	function handleDragEnd() {
		globalDraggedItem.set(null);
		globalDropTarget.set(null);
		document.body.classList.remove('dragging');
	}

	function handleDragOver(event: DragEvent, targetId: number, position: 'before' | 'after' | 'inside', targetCategory: number) {
		event.preventDefault();
		if (event.dataTransfer) {
			event.dataTransfer.dropEffect = 'move';
		}
		const dragged = $globalDraggedItem;
		// Don't allow dropping on itself
		if (dragged && dragged.id === targetId && dragged.type === 'category') {
			return;
		}
		globalDropTarget.set({ id: targetId, position, targetCategory });
	}

	function handleDragLeave(event: DragEvent) {
		// Only clear if we're actually leaving the element
		const relatedTarget = event.relatedTarget as HTMLElement;
		if (!relatedTarget || !event.currentTarget || !(event.currentTarget as HTMLElement).contains(relatedTarget)) {
			globalDropTarget.set(null);
		}
	}

	function handleDrop(event: DragEvent, targetCategoryId: number, position: number) {
		event.preventDefault();
		event.stopPropagation();
		const dragged = $globalDraggedItem;
		if (dragged) {
			// Don't drop category on itself
			if (dragged.type === 'category' && dragged.id === targetCategoryId) {
				globalDraggedItem.set(null);
				globalDropTarget.set(null);
				document.body.classList.remove('dragging');
				return;
			}
			dispatch('moveItem', {
				type: dragged.type,
				id: dragged.id,
				targetCategoryId,
				position
			});
		}
		globalDraggedItem.set(null);
		globalDropTarget.set(null);
		document.body.classList.remove('dragging');
	}

	function getIcon(icon: string | null): string {
		if (!icon) return '📁';
		// If it's an emoji (starts with emoji), return as-is
		if (icon.match(/^\p{Emoji}/u)) return icon;
		// Legacy string icons
		switch (icon) {
			case 'folder': return '📁';
			case 'code': return '💻';
			case 'book': return '📖';
			case 'globe': return '🌐';
			case 'star': return '⭐';
			default: return icon.length <= 2 ? icon : '📁';
		}
	}
</script>

<div class="tree-container">
	{#each nodes as node, index (node.id)}
		<div class="tree-node" animate:flip={{ duration: 200 }}>
			<div
				class="tree-item category-item"
				class:active={selectedCategoryId === node.id}
				class:drag-over={$globalDropTarget?.id === node.id && $globalDropTarget?.position === 'inside'}
				class:drag-before={$globalDropTarget?.id === node.id && $globalDropTarget?.position === 'before'}
				class:drag-after={$globalDropTarget?.id === node.id && $globalDropTarget?.position === 'after'}
				class:is-dragging={$globalDraggedItem?.type === 'category' && $globalDraggedItem?.id === node.id}
				draggable={depth > 0}
				on:click={() => handleCategoryClick(node.id)}
				on:dblclick={() => handleDoubleClick(node.id)}
				on:contextmenu={(e) => handleContextMenu(e, 'category', node.id)}
				on:dragstart={(e) => handleDragStart(e, 'category', node.id)}
				on:dragend={handleDragEnd}
				on:dragover|preventDefault={(e) => handleDragOver(e, node.id, 'inside', node.id)}
				on:dragleave={handleDragLeave}
				on:drop|preventDefault={(e) => handleDrop(e, node.id, 0)}
				role="button"
				tabindex="0"
				on:keydown={(e) => e.key === 'Enter' && handleCategoryClick(node.id)}
			>
				<span
					class="tree-expand"
					class:expanded={expandedNodes.has(node.id)}
					class:has-children={node.children.length > 0 || node.articles.length > 0}
					on:click|stopPropagation={() => toggleNode(node.id)}
					role="button"
					tabindex="0"
					on:keydown|stopPropagation={(e) => e.key === 'Enter' && toggleNode(node.id)}
				>
					▶
				</span>
				<span class="tree-icon folder-icon">{getIcon(node.icon)}</span>
				<span class="tree-label">{node.name}</span>
			</div>

			{#if expandedNodes.has(node.id)}
				<div class="tree-children" transition:slide={{ duration: 150 }}>
					{#each node.articles as article, articleIndex (article.id)}
						<div
							class="tree-item tree-article"
							class:active={selectedArticleId === article.id}
							class:drag-over={$globalDropTarget?.id === article.id}
							class:is-dragging={$globalDraggedItem?.type === 'article' && $globalDraggedItem?.id === article.id}
							style:background-color={article.color || 'transparent'}
							draggable={true}
							on:click={() => handleArticleClick(article)}
							on:contextmenu={(e) => handleContextMenu(e, 'article', article.id, node.id)}
							on:dragstart={(e) => handleDragStart(e, 'article', article.id, node.id)}
							on:dragend={handleDragEnd}
							on:dragover|preventDefault={(e) => handleDragOver(e, article.id, 'before', node.id)}
							on:dragleave={handleDragLeave}
							on:drop|preventDefault={(e) => handleDrop(e, node.id, articleIndex)}
							role="button"
							tabindex="0"
							on:keydown={(e) => e.key === 'Enter' && handleArticleClick(article)}
							animate:flip={{ duration: 200 }}
						>
							<span class="tree-expand"></span>
							{#if article.color}
								<span class="color-dot" style:background-color={article.color.replace('0.3', '0.8')}></span>
							{/if}
							<span class="tree-icon">📄</span>
							<span class="tree-label">{article.title}</span>
						</div>
					{/each}

					{#if node.children.length > 0}
						<svelte:self
							nodes={node.children}
							{selectedArticleId}
							{selectedCategoryId}
							depth={depth + 1}
							on:selectArticle
							on:selectCategory
							on:contextMenu
							on:createArticle
							on:moveItem
						/>
					{/if}
				</div>
			{/if}
		</div>
	{/each}
</div>

<style>
	.tree-container {
		padding: 0;
	}

	.tree-node {
		user-select: none;
	}

	.tree-item {
		display: flex;
		align-items: center;
		padding: 6px 8px;
		border-radius: 6px;
		cursor: pointer;
		transition: all 0.2s ease;
		border: 2px solid transparent;
		margin: 1px 0;
	}

	.tree-item:hover {
		background-color: var(--bg-hover);
	}

	.tree-item.active {
		background-color: var(--bg-active);
	}

	.tree-item.is-dragging {
		opacity: 0.5;
		transform: scale(0.98);
	}

	.tree-item.drag-over {
		background-color: rgba(124, 92, 191, 0.3);
		border-color: var(--accent);
		transform: scale(1.02);
	}

	.tree-item.drag-before {
		border-top-color: var(--accent);
		margin-top: 3px;
	}

	.tree-item.drag-before::before {
		content: '';
		position: absolute;
		left: 0;
		right: 0;
		top: -2px;
		height: 3px;
		background-color: var(--accent);
		border-radius: 2px;
	}

	.tree-item.drag-after {
		border-bottom-color: var(--accent);
		margin-bottom: 3px;
	}

	.category-item {
		font-weight: 500;
		position: relative;
	}

	.tree-icon {
		width: 22px;
		height: 22px;
		display: flex;
		align-items: center;
		justify-content: center;
		margin-right: 6px;
		font-size: 16px;
		flex-shrink: 0;
	}

	.color-dot {
		width: 8px;
		height: 8px;
		border-radius: 50%;
		margin-right: 6px;
		flex-shrink: 0;
	}

	.tree-expand {
		width: 16px;
		height: 16px;
		display: flex;
		align-items: center;
		justify-content: center;
		margin-right: 4px;
		color: var(--text-muted);
		font-size: 10px;
		transition: transform 0.2s ease;
		opacity: 0.3;
		flex-shrink: 0;
	}

	.tree-expand.has-children {
		opacity: 1;
	}

	.tree-expand.expanded {
		transform: rotate(90deg);
	}

	.tree-label {
		flex: 1;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.tree-children {
		margin-left: 20px;
	}

	.tree-article {
		color: var(--text-secondary);
		font-size: 13px;
		position: relative;
	}

	.tree-article:hover {
		color: var(--link-color);
	}

	/* Smooth dragging cursor */
	:global(body.dragging) {
		cursor: grabbing !important;
	}

	:global(body.dragging *) {
		cursor: grabbing !important;
	}
</style>
