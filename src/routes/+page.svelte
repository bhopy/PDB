<script lang="ts">
	import { onMount } from 'svelte';
	import '../app.css';
	import TreeNav from '$lib/components/TreeNav.svelte';
	import ArticleView from '$lib/components/ArticleView.svelte';
	import CategoryView from '$lib/components/CategoryView.svelte';
	import Editor from '$lib/components/Editor.svelte';
	import SearchBar from '$lib/components/SearchBar.svelte';
	import IconPicker from '$lib/components/IconPicker.svelte';
	import Toast from '$lib/components/Toast.svelte';
	import QuickSwitcher from '$lib/components/QuickSwitcher.svelte';
	import Breadcrumbs from '$lib/components/Breadcrumbs.svelte';
	import {
		tree,
		currentArticle,
		searchResults,
		loadTree,
		loadArticle,
		loadArticleBySlug,
		searchArticles,
		createArticle,
		updateArticle,
		deleteArticle,
		createCategory,
		updateCategory,
		deleteCategory,
		moveArticle,
		moveCategory,
		reorderArticle,
		reorderCategory
	} from '$lib/stores/db';
	import { toasts } from '$lib/stores/toast';
	import type { TreeNode } from '$lib/stores/db';

	let isEditing = false;
	let selectedArticleId: number | null = null;
	let selectedCategoryId: number | null = null;
	let newArticleCategoryId: number | null = null;
	let searchBar: SearchBar;
	let isSearching = false;
	let viewMode: 'article' | 'category' | 'none' = 'none';

	// Context menu state
	let contextMenu: {
		show: boolean;
		x: number;
		y: number;
		type: 'category' | 'article';
		id: number;
		parentId?: number;
	} = {
		show: false,
		x: 0,
		y: 0,
		type: 'category',
		id: 0
	};

	// Rename modal state
	let renameModal: {
		show: boolean;
		type: 'category' | 'article';
		id: number;
		name: string;
	} = {
		show: false,
		type: 'category',
		id: 0,
		name: ''
	};

	// Icon picker state
	let iconPicker: {
		show: boolean;
		categoryId: number;
		currentIcon: string;
	} = {
		show: false,
		categoryId: 0,
		currentIcon: ''
	};

	// Quick switcher state
	let showQuickSwitcher = false;

	// Find category by ID
	function findCategory(nodes: TreeNode[], id: number): TreeNode | null {
		for (const node of nodes) {
			if (node.id === id) return node;
			const found = findCategory(node.children, id);
			if (found) return found;
		}
		return null;
	}

	$: selectedCategory = selectedCategoryId ? findCategory($tree, selectedCategoryId) : null;

	onMount(async () => {
		await loadTree();
	});

	function handleKeydown(event: KeyboardEvent) {
		// Ctrl+K for search
		if (event.ctrlKey && event.key === 'k') {
			event.preventDefault();
			searchBar?.focus();
		}
		// Ctrl+P for quick switcher
		if (event.ctrlKey && event.key === 'p') {
			event.preventDefault();
			showQuickSwitcher = true;
		}
		// Ctrl+N for new article
		if (event.ctrlKey && event.key === 'n') {
			event.preventDefault();
			startNewArticle();
		}
		// Escape to close menus
		if (event.key === 'Escape') {
			contextMenu.show = false;
			renameModal.show = false;
			iconPicker.show = false;
			showQuickSwitcher = false;
		}
	}

	async function handleSelectArticle(event: CustomEvent<{ id: number; slug: string }>) {
		selectedArticleId = event.detail.id;
		viewMode = 'article';
		isEditing = false;
		await loadArticle(event.detail.id);
	}

	function handleSelectCategory(event: CustomEvent<{ id: number }>) {
		selectedCategoryId = event.detail.id;
		selectedArticleId = null;
		viewMode = 'category';
		currentArticle.set(null);
	}

	function handleContextMenu(
		event: CustomEvent<{
			event: MouseEvent;
			type: 'category' | 'article';
			id: number;
			parentId?: number;
		}>
	) {
		const { event: mouseEvent, type, id, parentId } = event.detail;
		contextMenu = {
			show: true,
			x: mouseEvent.clientX,
			y: mouseEvent.clientY,
			type,
			id,
			parentId
		};
	}

	function handleCreateArticle(event: CustomEvent<{ categoryId: number }>) {
		newArticleCategoryId = event.detail.categoryId;
		isEditing = true;
		viewMode = 'article';
		currentArticle.set(null);
	}

	function startNewArticle() {
		newArticleCategoryId = selectedCategoryId || 1;
		isEditing = true;
		viewMode = 'article';
		currentArticle.set(null);
	}

	async function handleSearch(event: CustomEvent<{ query: string }>) {
		isSearching = true;
		await searchArticles(event.detail.query);
		isSearching = false;
	}

	async function handleSearchSelect(event: CustomEvent<{ id: number; slug: string }>) {
		selectedArticleId = event.detail.id;
		viewMode = 'article';
		isEditing = false;
		await loadArticle(event.detail.id);
	}

	async function handleSave(
		event: CustomEvent<{
			id?: number;
			title: string;
			content: string;
			categoryId: number | null;
			sourceUrl: string;
			sourceType: string;
			color: string | null;
			tags: string[];
		}>
	) {
		const { id, title, content, categoryId, sourceUrl, sourceType, color, tags } = event.detail;

		if (id) {
			await updateArticle(id, title, content, categoryId, sourceUrl || null, sourceType, color, tags);
			toasts.success('Article updated');
		} else {
			const newId = await createArticle(
				title,
				content,
				categoryId,
				sourceUrl || null,
				sourceType,
				color,
				tags
			);
			if (newId) {
				selectedArticleId = newId;
				await loadArticle(newId);
				toasts.success('Article created');
			}
		}

		isEditing = false;
	}

	function handleCancel() {
		isEditing = false;
		if (!selectedArticleId) {
			viewMode = selectedCategoryId ? 'category' : 'none';
		}
	}

	async function handleDelete(event: CustomEvent<{ id: number }>) {
		await deleteArticle(event.detail.id);
		toasts.success('Article deleted');
		isEditing = false;
		selectedArticleId = null;
		viewMode = selectedCategoryId ? 'category' : 'none';
	}

	async function handleWikiLinkClick(event: CustomEvent<{ slug: string }>) {
		const article = await loadArticleBySlug(event.detail.slug);
		if (article) {
			selectedArticleId = article.article.id;
			viewMode = 'article';
		}
	}

	function handleEdit() {
		isEditing = true;
	}

	function handleCategoryNewArticle() {
		newArticleCategoryId = selectedCategoryId;
		isEditing = true;
		viewMode = 'article';
		currentArticle.set(null);
	}

	// Quick switcher selection
	async function handleQuickSwitcherSelect(event: CustomEvent<{ id: number; slug: string; type: 'article' | 'category' }>) {
		const { id, type } = event.detail;
		if (type === 'article') {
			selectedArticleId = id;
			selectedCategoryId = null;
			viewMode = 'article';
			isEditing = false;
			await loadArticle(id);
		} else {
			selectedCategoryId = id;
			selectedArticleId = null;
			viewMode = 'category';
			currentArticle.set(null);
		}
	}

	// Breadcrumb category selection
	function handleBreadcrumbSelect(event: CustomEvent<{ id: number }>) {
		selectedCategoryId = event.detail.id;
		selectedArticleId = null;
		viewMode = 'category';
		currentArticle.set(null);
	}

	// Handle drag and drop
	async function handleMoveItem(
		event: CustomEvent<{
			type: 'category' | 'article';
			id: number;
			targetCategoryId: number | null;
			position: number;
		}>
	) {
		const { type, id, targetCategoryId, position } = event.detail;

		if (type === 'article') {
			// Move the article to the target category
			await moveArticle(id, targetCategoryId);
			// Then reorder it to the correct position
			await reorderArticle(id, position);
			toasts.success('Article moved');
		} else {
			// Move the category to the new parent
			await moveCategory(id, targetCategoryId);
			// Then reorder it to the correct position
			await reorderCategory(id, position);
			toasts.success('Category moved');
		}
	}

	// Context menu actions
	async function contextMenuNewCategory() {
		const name = prompt('New category name:');
		if (name) {
			await createCategory(name, contextMenu.id, '📁');
			toasts.success('Category created');
		}
		contextMenu.show = false;
	}

	async function contextMenuNewArticle() {
		newArticleCategoryId = contextMenu.id;
		isEditing = true;
		viewMode = 'article';
		currentArticle.set(null);
		contextMenu.show = false;
	}

	function contextMenuRename() {
		let name = '';
		if (contextMenu.type === 'category') {
			const cat = findCategory($tree, contextMenu.id);
			name = cat?.name || '';
		} else {
			const findArticle = (nodes: TreeNode[]): string => {
				for (const node of nodes) {
					const article = node.articles.find((a) => a.id === contextMenu.id);
					if (article) return article.title;
					const found = findArticle(node.children);
					if (found) return found;
				}
				return '';
			};
			name = findArticle($tree);
		}

		renameModal = {
			show: true,
			type: contextMenu.type,
			id: contextMenu.id,
			name
		};
		contextMenu.show = false;
	}

	function contextMenuChangeIcon() {
		const cat = findCategory($tree, contextMenu.id);
		iconPicker = {
			show: true,
			categoryId: contextMenu.id,
			currentIcon: cat?.icon || '📁'
		};
		contextMenu.show = false;
	}

	async function handleIconSelect(event: CustomEvent<{ icon: string }>) {
		const cat = findCategory($tree, iconPicker.categoryId);
		if (cat) {
			await updateCategory(iconPicker.categoryId, cat.name, event.detail.icon);
			toasts.success('Icon updated');
		}
		iconPicker.show = false;
	}

	async function handleRename() {
		if (renameModal.type === 'category') {
			const cat = findCategory($tree, renameModal.id);
			await updateCategory(renameModal.id, renameModal.name, cat?.icon || null);
			toasts.success('Category renamed');
		} else {
			const article = await loadArticle(renameModal.id);
			if (article) {
				await updateArticle(
					renameModal.id,
					renameModal.name,
					article.article.content,
					article.article.category_id,
					article.article.source_url,
					article.article.source_type,
					article.article.color,
					article.tags
				);
				toasts.success('Article renamed');
			}
		}
		renameModal.show = false;
	}

	async function contextMenuDelete() {
		const confirmMessage =
			contextMenu.type === 'category'
				? 'Delete this category? Articles will be moved to uncategorized.'
				: 'Delete this article?';

		if (confirm(confirmMessage)) {
			if (contextMenu.type === 'category') {
				await deleteCategory(contextMenu.id);
				toasts.success('Category deleted');
				if (selectedCategoryId === contextMenu.id) {
					selectedCategoryId = null;
					viewMode = 'none';
				}
			} else {
				await deleteArticle(contextMenu.id);
				toasts.success('Article deleted');
				if (selectedArticleId === contextMenu.id) {
					selectedArticleId = null;
					currentArticle.set(null);
					viewMode = selectedCategoryId ? 'category' : 'none';
				}
			}
		}
		contextMenu.show = false;
	}

	function closeContextMenu() {
		contextMenu.show = false;
	}
</script>

<svelte:window on:keydown={handleKeydown} on:click={closeContextMenu} />

<div class="app">
	<div class="sidebar">
		<div class="sidebar-header">
			<span class="sidebar-title">Personal Database</span>
			<div class="sidebar-actions">
				<button class="btn-icon" title="New Article (Ctrl+N)" on:click={startNewArticle}>
					➕
				</button>
			</div>
		</div>
		<div class="tree-container">
			<TreeNav
				nodes={$tree}
				{selectedArticleId}
				{selectedCategoryId}
				on:selectArticle={handleSelectArticle}
				on:selectCategory={handleSelectCategory}
				on:contextMenu={handleContextMenu}
				on:createArticle={handleCreateArticle}
				on:moveItem={handleMoveItem}
			/>
		</div>
	</div>

	<div class="main-content">
		<div class="header">
			<SearchBar
				bind:this={searchBar}
				results={$searchResults}
				isLoading={isSearching}
				on:search={handleSearch}
				on:select={handleSearchSelect}
			/>
			<button class="btn btn-primary" on:click={startNewArticle}>
				➕ New Article
			</button>
		</div>

		<div class="content-area">
			<Breadcrumbs
				tree={$tree}
				currentArticle={$currentArticle}
				currentCategoryId={selectedCategoryId}
				on:selectCategory={handleBreadcrumbSelect}
			/>
			{#if isEditing}
				<Editor
					article={$currentArticle}
					categoryId={newArticleCategoryId}
					categories={$tree}
					on:save={handleSave}
					on:cancel={handleCancel}
					on:delete={handleDelete}
				/>
			{:else if viewMode === 'article' && $currentArticle}
				<ArticleView
					article={$currentArticle}
					on:edit={handleEdit}
					on:linkClick={handleWikiLinkClick}
				/>
			{:else if viewMode === 'category' && selectedCategory}
				<CategoryView
					category={selectedCategory}
					on:selectArticle={handleSelectArticle}
					on:newArticle={handleCategoryNewArticle}
				/>
			{:else}
				<ArticleView
					article={null}
					on:edit={handleEdit}
					on:linkClick={handleWikiLinkClick}
				/>
			{/if}
		</div>
	</div>
</div>

<!-- Context Menu -->
{#if contextMenu.show}
	<div
		class="context-menu"
		style="left: {contextMenu.x}px; top: {contextMenu.y}px;"
		on:click|stopPropagation
		role="menu"
	>
		{#if contextMenu.type === 'category'}
			<div class="context-menu-item" on:click={contextMenuNewArticle} role="menuitem">
				📄 New Article
			</div>
			<div class="context-menu-item" on:click={contextMenuNewCategory} role="menuitem">
				📁 New Subcategory
			</div>
			<div class="context-menu-item" on:click={contextMenuChangeIcon} role="menuitem">
				🎨 Change Icon
			</div>
			<div class="context-menu-separator"></div>
		{/if}
		<div class="context-menu-item" on:click={contextMenuRename} role="menuitem">
			✏️ Rename
		</div>
		<div class="context-menu-item danger" on:click={contextMenuDelete} role="menuitem">
			🗑️ Delete
		</div>
	</div>
{/if}

<!-- Rename Modal -->
{#if renameModal.show}
	<div class="modal-overlay" on:click={() => (renameModal.show = false)} role="dialog">
		<div class="modal" on:click|stopPropagation role="document">
			<div class="modal-header">
				<h3 class="modal-title">Rename {renameModal.type === 'category' ? 'Category' : 'Article'}</h3>
			</div>
			<div class="modal-body">
				<input
					type="text"
					bind:value={renameModal.name}
					on:keydown={(e) => e.key === 'Enter' && handleRename()}
					style="width: 100%;"
				/>
			</div>
			<div class="modal-actions">
				<button class="btn btn-secondary" on:click={() => (renameModal.show = false)}>Cancel</button>
				<button class="btn btn-primary" on:click={handleRename}>Rename</button>
			</div>
		</div>
	</div>
{/if}

<!-- Icon Picker -->
<IconPicker
	show={iconPicker.show}
	currentIcon={iconPicker.currentIcon}
	on:select={handleIconSelect}
	on:close={() => (iconPicker.show = false)}
/>

<!-- Quick Switcher -->
<QuickSwitcher
	show={showQuickSwitcher}
	tree={$tree}
	on:select={handleQuickSwitcherSelect}
	on:close={() => (showQuickSwitcher = false)}
/>

<!-- Toast Notifications -->
<Toast />

<style>
	.app {
		display: flex;
		height: 100vh;
		overflow: hidden;
	}

	.sidebar {
		width: 280px;
		background-color: var(--bg-secondary);
		border-right: 1px solid var(--border-color);
		display: flex;
		flex-direction: column;
		flex-shrink: 0;
	}

	.sidebar-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 12px 16px;
		border-bottom: 1px solid var(--border-color);
	}

	.sidebar-title {
		font-weight: 600;
		font-size: 16px;
	}

	.sidebar-actions {
		display: flex;
		gap: 4px;
	}

	.tree-container {
		flex: 1;
		overflow-y: auto;
		padding: 8px;
	}

	.main-content {
		flex: 1;
		display: flex;
		flex-direction: column;
		overflow: hidden;
	}

	.header {
		height: 56px;
		background-color: var(--bg-secondary);
		border-bottom: 1px solid var(--border-color);
		display: flex;
		align-items: center;
		padding: 0 16px;
		gap: 16px;
	}

	.content-area {
		flex: 1;
		overflow-y: auto;
		padding: 24px;
	}

	.btn-icon {
		padding: 6px;
		border-radius: 4px;
		font-size: 16px;
	}

	.btn-icon:hover {
		background-color: var(--bg-hover);
	}

	.context-menu {
		position: fixed;
		background-color: var(--bg-secondary);
		border: 1px solid var(--border-color);
		border-radius: 4px;
		padding: 4px 0;
		min-width: 180px;
		z-index: 1000;
		box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
	}

	.context-menu-item {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 8px 16px;
		cursor: pointer;
		transition: background-color 0.15s ease;
	}

	.context-menu-item:hover {
		background-color: var(--bg-hover);
	}

	.context-menu-item.danger {
		color: var(--error);
	}

	.context-menu-separator {
		height: 1px;
		background-color: var(--border-color);
		margin: 4px 0;
	}

	.modal-overlay {
		position: fixed;
		inset: 0;
		background-color: rgba(0, 0, 0, 0.6);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 1000;
	}

	.modal {
		background-color: var(--bg-secondary);
		border-radius: 8px;
		padding: 24px;
		min-width: 400px;
		max-width: 90vw;
	}

	.modal-header {
		margin-bottom: 16px;
	}

	.modal-title {
		font-size: 18px;
		font-weight: 600;
		margin: 0;
	}

	.modal-body {
		margin-bottom: 24px;
	}

	.modal-actions {
		display: flex;
		justify-content: flex-end;
		gap: 8px;
	}
</style>
