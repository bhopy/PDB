<script lang="ts">
	import { createEventDispatcher } from 'svelte';
	import { marked } from 'marked';
	import type { ArticleWithTags, TreeNode, Attachment } from '../stores/db';
	import { ARTICLE_COLORS, pickAndAddAttachment, deleteAttachment } from '../stores/db';

	export let article: ArticleWithTags | null = null;
	export let categoryId: number | null = null;
	export let categories: TreeNode[] = [];

	const dispatch = createEventDispatcher<{
		save: {
			id?: number;
			title: string;
			content: string;
			categoryId: number | null;
			sourceUrl: string;
			sourceType: string;
			color: string | null;
			tags: string[];
		};
		cancel: void;
		delete: { id: number };
	}>();

	let title = article?.article.title || '';
	let content = article?.article.content || '';
	let selectedCategoryId = article?.article.category_id || categoryId || 1;
	let sourceUrl = article?.article.source_url || '';
	let sourceType = article?.article.source_type || 'manual';
	let selectedColor = article?.article.color || null;
	let tagsInput = article?.tags.join(', ') || '';
	let showPreview = true;
	let attachments: Attachment[] = article?.attachments || [];
	let isAddingAttachment = false;

	$: preview = marked.parse(content) as string;
	$: isNew = !article;

	function flattenCategories(nodes: TreeNode[], prefix = ''): { id: number; name: string }[] {
		let result: { id: number; name: string }[] = [];
		for (const node of nodes) {
			result.push({ id: node.id, name: prefix + node.name });
			if (node.children.length > 0) {
				result = result.concat(flattenCategories(node.children, prefix + '  '));
			}
		}
		return result;
	}

	$: flatCategories = flattenCategories(categories);

	function handleSave() {
		if (!title.trim()) {
			alert('Title is required');
			return;
		}

		const tags = tagsInput
			.split(',')
			.map((t) => t.trim())
			.filter((t) => t.length > 0);

		dispatch('save', {
			id: article?.article.id,
			title: title.trim(),
			content,
			categoryId: selectedCategoryId,
			sourceUrl: sourceUrl.trim(),
			sourceType,
			color: selectedColor,
			tags
		});
	}

	async function handleAddAttachment() {
		if (!article?.article.id) {
			alert('Please save the article first before adding attachments');
			return;
		}

		isAddingAttachment = true;
		try {
			const attachment = await pickAndAddAttachment(article.article.id);
			if (attachment) {
				attachments = [...attachments, attachment];
			}
		} finally {
			isAddingAttachment = false;
		}
	}

	async function handleDeleteAttachment(id: number) {
		if (confirm('Delete this attachment?')) {
			const success = await deleteAttachment(id);
			if (success) {
				attachments = attachments.filter(a => a.id !== id);
			}
		}
	}

	function formatFileSize(bytes: number): string {
		if (bytes < 1024) return bytes + ' B';
		if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB';
		return (bytes / (1024 * 1024)).toFixed(1) + ' MB';
	}

	function getFileIcon(fileType: string): string {
		if (fileType.startsWith('image/')) return '🖼️';
		if (fileType.startsWith('video/')) return '🎬';
		if (fileType.startsWith('audio/')) return '🎵';
		if (fileType.includes('pdf')) return '📕';
		if (fileType.includes('zip') || fileType.includes('rar') || fileType.includes('7z')) return '📦';
		if (fileType.includes('text') || fileType.includes('document')) return '📝';
		if (fileType.includes('spreadsheet') || fileType.includes('excel')) return '📊';
		return '📎';
	}

	function handleDelete() {
		if (article && confirm('Are you sure you want to delete this article?')) {
			dispatch('delete', { id: article.article.id });
		}
	}

	function handleKeydown(event: KeyboardEvent) {
		if (event.ctrlKey && event.key === 's') {
			event.preventDefault();
			handleSave();
		}
		if (event.key === 'Escape') {
			dispatch('cancel');
		}
	}

	function insertMarkdown(before: string, after: string = '') {
		const textarea = document.querySelector('.editor-textarea') as HTMLTextAreaElement;
		if (!textarea) return;

		const start = textarea.selectionStart;
		const end = textarea.selectionEnd;
		const selectedText = content.substring(start, end);

		content = content.substring(0, start) + before + selectedText + after + content.substring(end);

		// Restore cursor position
		setTimeout(() => {
			textarea.focus();
			const newCursorPos = start + before.length + selectedText.length;
			textarea.setSelectionRange(newCursorPos, newCursorPos);
		}, 0);
	}
</script>

<svelte:window on:keydown={handleKeydown} />

<div class="editor">
	<div class="editor-header">
		<input
			type="text"
			class="editor-title-input"
			placeholder="Article Title"
			bind:value={title}
		/>
	</div>

	<div class="editor-toolbar">
		<button class="btn-icon" title="Bold (Ctrl+B)" on:click={() => insertMarkdown('**', '**')}>
			<strong>B</strong>
		</button>
		<button class="btn-icon" title="Italic (Ctrl+I)" on:click={() => insertMarkdown('*', '*')}>
			<em>I</em>
		</button>
		<button class="btn-icon" title="Code" on:click={() => insertMarkdown('`', '`')}>
			<code>&lt;/&gt;</code>
		</button>
		<button class="btn-icon" title="Link" on:click={() => insertMarkdown('[', '](url)')}>
			&#128279;
		</button>
		<button class="btn-icon" title="Wiki Link" on:click={() => insertMarkdown('[[', ']]')}>
			&#128214;
		</button>
		<button class="btn-icon" title="Heading" on:click={() => insertMarkdown('## ')}>
			H
		</button>
		<button class="btn-icon" title="List" on:click={() => insertMarkdown('- ')}>
			&#8226;
		</button>
		<button class="btn-icon" title="Code Block" on:click={() => insertMarkdown('```\n', '\n```')}>
			&#9635;
		</button>
		<div style="flex: 1;"></div>
		<button
			class="btn btn-secondary"
			class:active={showPreview}
			on:click={() => (showPreview = !showPreview)}
		>
			{showPreview ? 'Hide Preview' : 'Show Preview'}
		</button>
	</div>

	<div class="editor-content">
		<textarea
			class="editor-textarea"
			placeholder="Write your content in Markdown..."
			bind:value={content}
		></textarea>

		{#if showPreview}
			<div class="editor-preview article-content">
				{@html preview}
			</div>
		{/if}
	</div>

	<div class="editor-meta">
		<div class="editor-meta-field">
			<label for="category">Category</label>
			<select id="category" bind:value={selectedCategoryId}>
				{#each flatCategories as cat}
					<option value={cat.id}>{cat.name}</option>
				{/each}
			</select>
		</div>

		<div class="editor-meta-field">
			<label for="source-type">Source Type</label>
			<select id="source-type" bind:value={sourceType}>
				<option value="manual">Manual</option>
				<option value="web">Web</option>
				<option value="code">Code</option>
				<option value="mcp_analysis">MCP Analysis</option>
			</select>
		</div>

		<div class="editor-meta-field">
			<label for="color">Color Label</label>
			<div class="color-picker">
				<select id="color" bind:value={selectedColor} class="color-select" style:background-color={selectedColor || 'transparent'}>
					{#each ARTICLE_COLORS as color}
						<option value={color.value} style:background-color={color.value || 'var(--bg-secondary)'}>{color.name}</option>
					{/each}
				</select>
				{#if selectedColor}
					<span class="color-preview" style:background-color={selectedColor}></span>
				{/if}
			</div>
		</div>

		<div class="editor-meta-field" style="flex: 2;">
			<label for="source-url">Source URL</label>
			<input
				type="url"
				id="source-url"
				placeholder="https://..."
				bind:value={sourceUrl}
			/>
		</div>
	</div>

	<div class="editor-meta">
		<div class="editor-meta-field" style="flex: 1;">
			<label for="tags">Tags (comma-separated)</label>
			<input
				type="text"
				id="tags"
				placeholder="tag1, tag2, tag3"
				bind:value={tagsInput}
			/>
		</div>
	</div>

	{#if !isNew}
		<div class="editor-attachments">
			<div class="attachments-header">
				<label>Attachments</label>
				<button class="btn btn-secondary btn-sm" on:click={handleAddAttachment} disabled={isAddingAttachment}>
					{isAddingAttachment ? 'Adding...' : '+ Add File'}
				</button>
			</div>
			{#if attachments.length > 0}
				<div class="attachments-list">
					{#each attachments as attachment}
						<div class="attachment-item">
							<span class="attachment-icon">{getFileIcon(attachment.file_type)}</span>
							<span class="attachment-name" title={attachment.original_name}>{attachment.original_name}</span>
							<span class="attachment-size">{formatFileSize(attachment.file_size)}</span>
							<button class="btn-icon btn-delete" title="Delete" on:click={() => handleDeleteAttachment(attachment.id)}>
								&times;
							</button>
						</div>
					{/each}
				</div>
			{:else}
				<p class="no-attachments">No attachments yet. Click "Add File" to attach files.</p>
			{/if}
		</div>
	{/if}

	<div class="editor-actions">
		{#if !isNew}
			<button class="btn btn-danger" on:click={handleDelete}>
				Delete
			</button>
		{/if}
		<div style="flex: 1;"></div>
		<button class="btn btn-secondary" on:click={() => dispatch('cancel')}>
			Cancel
		</button>
		<button class="btn btn-primary" on:click={handleSave}>
			{isNew ? 'Create' : 'Save'}
		</button>
	</div>
</div>

<style>
	.editor {
		display: flex;
		flex-direction: column;
		height: 100%;
		max-width: 1400px;
		margin: 0 auto;
		padding: 0 16px;
	}

	.editor-header {
		margin-bottom: 16px;
	}

	.editor-title-input {
		width: 100%;
		font-size: 24px;
		font-weight: 600;
		background: transparent;
		border: none;
		border-bottom: 2px solid var(--border-color);
		border-radius: 0;
		padding: 8px 0;
		color: var(--text-primary);
	}

	.editor-title-input:focus {
		outline: none;
		border-color: var(--accent);
	}

	.editor-toolbar {
		display: flex;
		gap: 4px;
		padding: 8px;
		background-color: var(--bg-secondary);
		border-radius: 4px;
		margin-bottom: 16px;
		flex-wrap: wrap;
	}

	.btn-icon {
		padding: 6px 10px;
		border-radius: 4px;
		font-size: 14px;
		min-width: 32px;
	}

	.btn-icon:hover {
		background-color: var(--bg-hover);
	}

	.editor-content {
		display: flex;
		gap: 16px;
		flex: 1;
		min-height: 0;
	}

	.editor-textarea {
		flex: 1;
		font-family: var(--font-mono);
		font-size: 14px;
		line-height: 1.6;
		resize: none;
		padding: 16px;
		background-color: var(--bg-tertiary);
		border: 1px solid var(--border-color);
		border-radius: 4px;
		color: var(--text-primary);
	}

	.editor-textarea:focus {
		outline: none;
		border-color: var(--accent);
	}

	.editor-preview {
		flex: 1;
		overflow-y: auto;
		padding: 16px;
		background-color: var(--bg-secondary);
		border-radius: 4px;
		border: 1px solid var(--border-color);
	}

	.editor-meta {
		display: flex;
		gap: 16px;
		padding: 16px 0;
		border-top: 1px solid var(--border-color);
		flex-wrap: wrap;
	}

	.editor-meta:first-of-type {
		margin-top: 16px;
	}

	.editor-meta-field {
		flex: 1;
		min-width: 150px;
	}

	.editor-meta-field label {
		display: block;
		margin-bottom: 4px;
		color: var(--text-secondary);
		font-size: 12px;
	}

	.editor-meta-field input,
	.editor-meta-field select {
		width: 100%;
	}

	.editor-actions {
		display: flex;
		justify-content: flex-end;
		gap: 8px;
		padding-top: 16px;
		border-top: 1px solid var(--border-color);
	}

	.btn.active {
		background-color: var(--accent);
		color: white;
	}

	/* Color picker styles */
	.color-picker {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.color-select {
		flex: 1;
		border-radius: 4px;
	}

	.color-preview {
		width: 24px;
		height: 24px;
		border-radius: 4px;
		border: 1px solid var(--border-color);
		flex-shrink: 0;
	}

	/* Attachments styles */
	.editor-attachments {
		padding: 16px 0;
		border-top: 1px solid var(--border-color);
	}

	.attachments-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 12px;
	}

	.attachments-header label {
		color: var(--text-secondary);
		font-size: 12px;
		text-transform: uppercase;
		letter-spacing: 0.5px;
	}

	.btn-sm {
		padding: 4px 10px;
		font-size: 12px;
	}

	.attachments-list {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.attachment-item {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 10px 12px;
		background-color: var(--bg-secondary);
		border-radius: 6px;
		border: 1px solid var(--border-color);
		transition: background-color 0.2s ease;
	}

	.attachment-item:hover {
		background-color: var(--bg-hover);
	}

	.attachment-icon {
		font-size: 18px;
		flex-shrink: 0;
	}

	.attachment-name {
		flex: 1;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		color: var(--text-primary);
	}

	.attachment-size {
		color: var(--text-muted);
		font-size: 12px;
		flex-shrink: 0;
	}

	.btn-delete {
		width: 24px;
		height: 24px;
		font-size: 16px;
		color: var(--text-muted);
		border-radius: 4px;
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.btn-delete:hover {
		background-color: rgba(239, 68, 68, 0.2);
		color: #ef4444;
	}

	.no-attachments {
		color: var(--text-muted);
		font-size: 13px;
		font-style: italic;
		text-align: center;
		padding: 16px;
		background-color: var(--bg-secondary);
		border-radius: 6px;
		margin: 0;
	}
</style>
