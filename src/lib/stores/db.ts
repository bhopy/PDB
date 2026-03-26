import { writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';

// Types matching Rust structures
export interface TreeNode {
	id: number;
	name: string;
	icon: string | null;
	children: TreeNode[];
	articles: ArticleSummary[];
}

export interface ArticleSummary {
	id: number;
	title: string;
	slug: string;
	color: string | null;
	sort_order: number;
}

export interface Article {
	id: number;
	title: string;
	slug: string;
	content: string | null;
	category_id: number | null;
	source_url: string | null;
	source_type: string | null;
	color: string | null;
	sort_order: number;
	created_at: string;
	updated_at: string;
}

export interface Attachment {
	id: number;
	article_id: number;
	filename: string;
	original_name: string;
	file_type: string;
	file_size: number;
	created_at: string;
}

export interface ArticleWithTags {
	article: Article;
	tags: string[];
	attachments: Attachment[];
}

export interface SearchResult {
	id: number;
	title: string;
	slug: string;
	snippet: string;
}

export interface Tag {
	id: number;
	name: string;
}

// Soft/muted color palette for article labels
export const ARTICLE_COLORS = [
	{ name: 'None', value: null },
	{ name: 'Red', value: 'rgba(239, 68, 68, 0.3)' },
	{ name: 'Orange', value: 'rgba(249, 115, 22, 0.3)' },
	{ name: 'Amber', value: 'rgba(245, 158, 11, 0.3)' },
	{ name: 'Yellow', value: 'rgba(234, 179, 8, 0.3)' },
	{ name: 'Lime', value: 'rgba(132, 204, 22, 0.3)' },
	{ name: 'Green', value: 'rgba(34, 197, 94, 0.3)' },
	{ name: 'Teal', value: 'rgba(20, 184, 166, 0.3)' },
	{ name: 'Cyan', value: 'rgba(6, 182, 212, 0.3)' },
	{ name: 'Blue', value: 'rgba(59, 130, 246, 0.3)' },
	{ name: 'Purple', value: 'rgba(139, 92, 246, 0.3)' },
	{ name: 'Pink', value: 'rgba(236, 72, 153, 0.3)' },
	{ name: 'Gray', value: 'rgba(107, 114, 128, 0.3)' }
];

// Stores
export const tree = writable<TreeNode[]>([]);
export const currentArticle = writable<ArticleWithTags | null>(null);
export const searchResults = writable<SearchResult[]>([]);
export const allTags = writable<Tag[]>([]);
export const isLoading = writable(false);
export const error = writable<string | null>(null);

// API functions
export async function loadTree(): Promise<void> {
	try {
		isLoading.set(true);
		const result = await invoke<TreeNode[]>('get_tree');
		tree.set(result);
	} catch (e) {
		error.set(String(e));
		console.error('Failed to load tree:', e);
	} finally {
		isLoading.set(false);
	}
}

export async function loadArticle(id: number): Promise<ArticleWithTags | null> {
	try {
		isLoading.set(true);
		const result = await invoke<ArticleWithTags | null>('get_article', { id });
		currentArticle.set(result);
		return result;
	} catch (e) {
		error.set(String(e));
		console.error('Failed to load article:', e);
		return null;
	} finally {
		isLoading.set(false);
	}
}

export async function loadArticleBySlug(slug: string): Promise<ArticleWithTags | null> {
	try {
		isLoading.set(true);
		const result = await invoke<ArticleWithTags | null>('get_article_by_slug', { slug });
		currentArticle.set(result);
		return result;
	} catch (e) {
		error.set(String(e));
		console.error('Failed to load article by slug:', e);
		return null;
	} finally {
		isLoading.set(false);
	}
}

export async function searchArticles(query: string): Promise<SearchResult[]> {
	try {
		const result = await invoke<SearchResult[]>('search', { query });
		searchResults.set(result);
		return result;
	} catch (e) {
		error.set(String(e));
		console.error('Failed to search:', e);
		return [];
	}
}

export async function createArticle(
	title: string,
	content: string | null,
	categoryId: number | null,
	sourceUrl: string | null,
	sourceType: string | null,
	color: string | null,
	tags: string[]
): Promise<number | null> {
	try {
		isLoading.set(true);
		const id = await invoke<number>('create_article', {
			title,
			content,
			categoryId,
			sourceUrl,
			sourceType,
			color,
			tags
		});
		await loadTree();
		return id;
	} catch (e) {
		error.set(String(e));
		console.error('Failed to create article:', e);
		return null;
	} finally {
		isLoading.set(false);
	}
}

export async function updateArticle(
	id: number,
	title: string,
	content: string | null,
	categoryId: number | null,
	sourceUrl: string | null,
	sourceType: string | null,
	color: string | null,
	tags: string[]
): Promise<boolean> {
	try {
		isLoading.set(true);
		await invoke('update_article', {
			id,
			title,
			content,
			categoryId,
			sourceUrl,
			sourceType,
			color,
			tags
		});
		await loadTree();
		await loadArticle(id);
		return true;
	} catch (e) {
		error.set(String(e));
		console.error('Failed to update article:', e);
		return false;
	} finally {
		isLoading.set(false);
	}
}

export async function deleteArticle(id: number): Promise<boolean> {
	try {
		isLoading.set(true);
		await invoke('delete_article', { id });
		currentArticle.set(null);
		await loadTree();
		return true;
	} catch (e) {
		error.set(String(e));
		console.error('Failed to delete article:', e);
		return false;
	} finally {
		isLoading.set(false);
	}
}

export async function moveArticle(id: number, categoryId: number | null): Promise<boolean> {
	try {
		await invoke('move_article', { id, categoryId });
		await loadTree();
		return true;
	} catch (e) {
		error.set(String(e));
		console.error('Failed to move article:', e);
		return false;
	}
}

export async function reorderArticle(id: number, newSortOrder: number): Promise<boolean> {
	try {
		await invoke('reorder_article', { id, newSortOrder });
		await loadTree();
		return true;
	} catch (e) {
		error.set(String(e));
		console.error('Failed to reorder article:', e);
		return false;
	}
}

export async function createCategory(
	name: string,
	parentId: number | null,
	icon: string | null
): Promise<number | null> {
	try {
		isLoading.set(true);
		const id = await invoke<number>('create_category', { name, parentId, icon });
		await loadTree();
		return id;
	} catch (e) {
		error.set(String(e));
		console.error('Failed to create category:', e);
		return null;
	} finally {
		isLoading.set(false);
	}
}

export async function updateCategory(
	id: number,
	name: string,
	icon: string | null
): Promise<boolean> {
	try {
		isLoading.set(true);
		await invoke('update_category', { id, name, icon });
		await loadTree();
		return true;
	} catch (e) {
		error.set(String(e));
		console.error('Failed to update category:', e);
		return false;
	} finally {
		isLoading.set(false);
	}
}

export async function moveCategory(id: number, newParentId: number | null): Promise<boolean> {
	try {
		await invoke('move_category', { id, newParentId });
		await loadTree();
		return true;
	} catch (e) {
		error.set(String(e));
		console.error('Failed to move category:', e);
		return false;
	}
}

export async function reorderCategory(id: number, newSortOrder: number): Promise<boolean> {
	try {
		await invoke('reorder_category', { id, newSortOrder });
		await loadTree();
		return true;
	} catch (e) {
		error.set(String(e));
		console.error('Failed to reorder category:', e);
		return false;
	}
}

export async function deleteCategory(id: number): Promise<boolean> {
	try {
		isLoading.set(true);
		await invoke('delete_category', { id });
		await loadTree();
		return true;
	} catch (e) {
		error.set(String(e));
		console.error('Failed to delete category:', e);
		return false;
	} finally {
		isLoading.set(false);
	}
}

export async function loadAllTags(): Promise<Tag[]> {
	try {
		const result = await invoke<Tag[]>('get_all_tags');
		allTags.set(result);
		return result;
	} catch (e) {
		error.set(String(e));
		console.error('Failed to load tags:', e);
		return [];
	}
}

export async function getArticlesByTag(tagName: string): Promise<ArticleSummary[]> {
	try {
		const result = await invoke<ArticleSummary[]>('get_articles_by_tag', { tagName });
		return result;
	} catch (e) {
		error.set(String(e));
		console.error('Failed to get articles by tag:', e);
		return [];
	}
}

// Attachment functions
export async function pickAndAddAttachment(articleId: number): Promise<Attachment | null> {
	try {
		const selected = await open({
			multiple: false,
			title: 'Select file to attach'
		});

		if (selected && typeof selected === 'string') {
			const originalName = selected.split(/[/\\]/).pop() || 'file';
			const attachment = await invoke<Attachment>('add_attachment', {
				articleId,
				sourcePath: selected,
				originalName
			});
			return attachment;
		}
		return null;
	} catch (e) {
		error.set(String(e));
		console.error('Failed to add attachment:', e);
		return null;
	}
}

export async function getAttachments(articleId: number): Promise<Attachment[]> {
	try {
		const result = await invoke<Attachment[]>('get_attachments', { articleId });
		return result;
	} catch (e) {
		error.set(String(e));
		console.error('Failed to get attachments:', e);
		return [];
	}
}

export async function deleteAttachment(id: number): Promise<boolean> {
	try {
		await invoke('delete_attachment', { id });
		return true;
	} catch (e) {
		error.set(String(e));
		console.error('Failed to delete attachment:', e);
		return false;
	}
}

// Helper to get attachment URL for display
export function getAttachmentPath(filename: string): string {
	return `E:\\PDB\\attachments\\${filename}`;
}
