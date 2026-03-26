<script lang="ts">
	import { createEventDispatcher, onMount } from 'svelte';
	import { marked } from 'marked';
	import type { ArticleWithTags, Attachment } from '../stores/db';
	import { getAttachmentPath } from '../stores/db';

	export let article: ArticleWithTags | null = null;

	const dispatch = createEventDispatcher<{
		edit: void;
		linkClick: { slug: string };
	}>();

	let renderedContent = '';
	let tableOfContents: { id: string; text: string; level: number }[] = [];
	let activeHeadingId = '';
	let contentEl: HTMLElement;

	// Configure marked for wiki-style links and heading IDs
	const renderer = new marked.Renderer();
	const originalLinkRenderer = renderer.link.bind(renderer);

	renderer.link = function (href, title, text) {
		// Handle wiki-style links [[Article Name]]
		if (href && href.startsWith('wiki:')) {
			const slug = href.replace('wiki:', '');
			return `<a href="#" class="wiki-link" data-slug="${slug}">${text}</a>`;
		}
		return originalLinkRenderer(href, title, text);
	};

	// Add IDs to headings for TOC linking
	renderer.heading = function (text, level) {
		const id = text.toLowerCase().replace(/[^\w]+/g, '-');
		return `<h${level} id="${id}">${text}</h${level}>`;
	};

	marked.setOptions({
		renderer,
		gfm: true,
		breaks: true
	});

	function processWikiLinks(content: string): string {
		// Convert [[Article Name]] to [Article Name](wiki:article-name)
		return content.replace(/\[\[([^\]]+)\]\]/g, (match, title) => {
			const slug = title.toLowerCase().replace(/\s+/g, '-');
			return `[${title}](wiki:${slug})`;
		});
	}

	// Extract headings for TOC
	function extractHeadings(content: string): { id: string; text: string; level: number }[] {
		const headings: { id: string; text: string; level: number }[] = [];
		const regex = /^(#{1,4})\s+(.+)$/gm;
		let match;

		while ((match = regex.exec(content)) !== null) {
			const level = match[1].length;
			const text = match[2].trim();
			const id = text.toLowerCase().replace(/[^\w]+/g, '-');
			headings.push({ id, text, level });
		}

		return headings;
	}

	// Calculate reading time
	function getReadingTime(content: string): { minutes: number; words: number } {
		const text = content.replace(/[#*`\[\]()]/g, '');
		const words = text.split(/\s+/).filter(w => w.length > 0).length;
		const minutes = Math.max(1, Math.ceil(words / 200));
		return { minutes, words };
	}

	$: if (article?.article.content) {
		const processed = processWikiLinks(article.article.content);
		renderedContent = marked.parse(processed) as string;
		tableOfContents = extractHeadings(article.article.content);
	} else {
		renderedContent = '';
		tableOfContents = [];
	}

	$: readingStats = article?.article.content ? getReadingTime(article.article.content) : null;

	function scrollToHeading(id: string) {
		const el = document.getElementById(id);
		if (el) {
			el.scrollIntoView({ behavior: 'smooth', block: 'start' });
			activeHeadingId = id;
		}
	}

	function handleContentClick(event: MouseEvent) {
		const target = event.target as HTMLElement;
		if (target.classList.contains('wiki-link')) {
			event.preventDefault();
			const slug = target.getAttribute('data-slug');
			if (slug) {
				dispatch('linkClick', { slug });
			}
		}
	}

	function formatDate(dateString: string): string {
		const date = new Date(dateString);
		return date.toLocaleDateString('en-US', {
			year: 'numeric',
			month: 'long',
			day: 'numeric',
			hour: '2-digit',
			minute: '2-digit'
		});
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
</script>

{#if article}
	<div class="article-layout">
		<div class="article">
			<div class="article-header">
				<div class="article-header-top">
					<h1 class="article-title">{article.article.title}</h1>
					<button class="btn btn-secondary" on:click={() => dispatch('edit')}>
						Edit
					</button>
				</div>
				<div class="article-meta">
					{#if readingStats}
						<span class="reading-time">{readingStats.minutes} min read</span>
						<span class="word-count">{readingStats.words.toLocaleString()} words</span>
					{/if}
					<span>Created: {formatDate(article.article.created_at)}</span>
					<span>Updated: {formatDate(article.article.updated_at)}</span>
					{#if article.article.source_type}
						<span class="source-type">{article.article.source_type}</span>
					{/if}
				</div>
				{#if article.tags.length > 0}
					<div class="tags">
						{#each article.tags as tag}
							<span class="tag">{tag}</span>
						{/each}
					</div>
				{/if}
				{#if article.article.source_url}
					<div class="source-url">
						Source: <a href={article.article.source_url} target="_blank" rel="noopener">{article.article.source_url}</a>
					</div>
				{/if}
			</div>

			<div class="article-content" bind:this={contentEl} on:click={handleContentClick} role="article">
				{@html renderedContent}
			</div>

			{#if article.attachments && article.attachments.length > 0}
				<div class="attachments-section">
					<h3 class="attachments-title">Attachments ({article.attachments.length})</h3>
					<div class="attachments-grid">
						{#each article.attachments as attachment}
							<div class="attachment-card">
								<span class="attachment-icon">{getFileIcon(attachment.file_type)}</span>
								<div class="attachment-info">
									<span class="attachment-name" title={attachment.original_name}>{attachment.original_name}</span>
									<span class="attachment-meta">{formatFileSize(attachment.file_size)}</span>
								</div>
							</div>
						{/each}
					</div>
				</div>
			{/if}
		</div>

		{#if tableOfContents.length > 1}
			<aside class="toc">
				<div class="toc-header">On this page</div>
				<nav class="toc-nav">
					{#each tableOfContents as heading}
						<button
							class="toc-item toc-level-{heading.level}"
							class:active={activeHeadingId === heading.id}
							on:click={() => scrollToHeading(heading.id)}
						>
							{heading.text}
						</button>
					{/each}
				</nav>
			</aside>
		{/if}
	</div>
{:else}
	<div class="empty-state">
		<div class="empty-state-icon">&#128214;</div>
		<div class="empty-state-title">No Article Selected</div>
		<div class="empty-state-description">
			Select an article from the tree or create a new one to get started.
			<br><br>
			<kbd>Ctrl</kbd> + <kbd>P</kbd> quick switcher<br>
			<kbd>Ctrl</kbd> + <kbd>K</kbd> to search<br>
			<kbd>Ctrl</kbd> + <kbd>N</kbd> new article
		</div>
	</div>
{/if}

<style>
	.article-layout {
		display: flex;
		gap: 32px;
		max-width: 1200px;
		margin: 0 auto;
	}

	.article {
		flex: 1;
		min-width: 0;
		max-width: 900px;
	}

	.toc {
		width: 220px;
		flex-shrink: 0;
		position: sticky;
		top: 0;
		max-height: calc(100vh - 120px);
		overflow-y: auto;
	}

	.toc-header {
		font-size: 12px;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.05em;
		color: var(--text-muted);
		margin-bottom: 12px;
		padding-bottom: 8px;
		border-bottom: 1px solid var(--border-color);
	}

	.toc-nav {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.toc-item {
		display: block;
		text-align: left;
		background: none;
		border: none;
		padding: 6px 12px;
		font-size: 13px;
		color: var(--text-secondary);
		cursor: pointer;
		border-radius: 4px;
		transition: all 0.15s;
		border-left: 2px solid transparent;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.toc-item:hover {
		color: var(--text-primary);
		background-color: var(--bg-hover);
	}

	.toc-item.active {
		color: var(--accent);
		border-left-color: var(--accent);
		background-color: rgba(124, 92, 191, 0.1);
	}

	.toc-level-1 { padding-left: 12px; }
	.toc-level-2 { padding-left: 24px; }
	.toc-level-3 { padding-left: 36px; }
	.toc-level-4 { padding-left: 48px; }

	.reading-time,
	.word-count {
		color: var(--accent);
		font-weight: 500;
	}

	@media (max-width: 1100px) {
		.toc {
			display: none;
		}
	}

	.article-header {
		margin-bottom: 24px;
		padding-bottom: 16px;
		border-bottom: 1px solid var(--border-color);
	}

	.article-header-top {
		display: flex;
		align-items: flex-start;
		justify-content: space-between;
		gap: 16px;
		margin-bottom: 12px;
	}

	.article-title {
		font-size: 28px;
		font-weight: 600;
		margin: 0;
		flex: 1;
	}

	.article-meta {
		display: flex;
		flex-wrap: wrap;
		gap: 16px;
		color: var(--text-secondary);
		font-size: 13px;
		margin-bottom: 12px;
	}

	.source-type {
		padding: 2px 8px;
		background-color: var(--bg-hover);
		border-radius: 4px;
		text-transform: capitalize;
	}

	.tags {
		display: flex;
		flex-wrap: wrap;
		gap: 6px;
		margin-bottom: 12px;
	}

	.tag {
		display: inline-block;
		padding: 2px 10px;
		background-color: var(--bg-hover);
		border-radius: 12px;
		font-size: 12px;
		color: var(--text-secondary);
	}

	.source-url {
		font-size: 13px;
		color: var(--text-secondary);
	}

	.source-url a {
		color: var(--link-color);
	}

	.article-content {
		line-height: 1.7;
	}

	.article-content :global(h1),
	.article-content :global(h2),
	.article-content :global(h3),
	.article-content :global(h4) {
		margin-top: 24px;
		margin-bottom: 16px;
		font-weight: 600;
	}

	.article-content :global(h1) { font-size: 24px; }
	.article-content :global(h2) { font-size: 20px; }
	.article-content :global(h3) { font-size: 18px; }
	.article-content :global(h4) { font-size: 16px; }

	.article-content :global(p) {
		margin-bottom: 16px;
	}

	.article-content :global(ul),
	.article-content :global(ol) {
		margin-bottom: 16px;
		padding-left: 24px;
	}

	.article-content :global(li) {
		margin-bottom: 4px;
	}

	.article-content :global(code) {
		font-family: var(--font-mono);
		background-color: var(--bg-tertiary);
		padding: 2px 6px;
		border-radius: 3px;
		font-size: 13px;
	}

	.article-content :global(pre) {
		background-color: var(--bg-tertiary);
		padding: 16px;
		border-radius: 6px;
		overflow-x: auto;
		margin-bottom: 16px;
	}

	.article-content :global(pre code) {
		background: none;
		padding: 0;
	}

	.article-content :global(blockquote) {
		border-left: 3px solid var(--accent);
		padding-left: 16px;
		margin: 16px 0;
		color: var(--text-secondary);
	}

	.article-content :global(table) {
		width: 100%;
		border-collapse: collapse;
		margin-bottom: 16px;
	}

	.article-content :global(th),
	.article-content :global(td) {
		border: 1px solid var(--border-color);
		padding: 8px 16px;
		text-align: left;
	}

	.article-content :global(th) {
		background-color: var(--bg-secondary);
	}

	.article-content :global(.wiki-link) {
		color: var(--link-color);
		cursor: pointer;
	}

	.article-content :global(.wiki-link:hover) {
		text-decoration: underline;
	}

	.empty-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		height: 100%;
		color: var(--text-muted);
		text-align: center;
		padding: 32px;
	}

	.empty-state-icon {
		font-size: 48px;
		margin-bottom: 16px;
		opacity: 0.5;
	}

	.empty-state-title {
		font-size: 18px;
		font-weight: 500;
		margin-bottom: 8px;
		color: var(--text-secondary);
	}

	.empty-state-description {
		max-width: 400px;
		line-height: 1.8;
	}

	.empty-state-description :global(kbd) {
		display: inline-block;
		padding: 2px 6px;
		font-family: var(--font-mono);
		font-size: 12px;
		background-color: var(--bg-tertiary);
		border: 1px solid var(--border-color);
		border-radius: 3px;
		color: var(--text-secondary);
	}

	/* Attachments section */
	.attachments-section {
		margin-top: 32px;
		padding-top: 24px;
		border-top: 1px solid var(--border-color);
	}

	.attachments-title {
		font-size: 16px;
		font-weight: 600;
		margin: 0 0 16px 0;
		color: var(--text-secondary);
	}

	.attachments-grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(250px, 1fr));
		gap: 12px;
	}

	.attachment-card {
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 12px 16px;
		background-color: var(--bg-secondary);
		border: 1px solid var(--border-color);
		border-radius: 8px;
		transition: all 0.2s ease;
		cursor: pointer;
	}

	.attachment-card:hover {
		background-color: var(--bg-hover);
		border-color: var(--accent);
	}

	.attachment-icon {
		font-size: 24px;
		flex-shrink: 0;
	}

	.attachment-info {
		flex: 1;
		min-width: 0;
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.attachment-name {
		font-size: 14px;
		color: var(--text-primary);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.attachment-meta {
		font-size: 12px;
		color: var(--text-muted);
	}
</style>
