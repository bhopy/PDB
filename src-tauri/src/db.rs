use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Mutex;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Category {
    pub id: i64,
    pub name: String,
    pub parent_id: Option<i64>,
    pub icon: Option<String>,
    pub sort_order: i32,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Article {
    pub id: i64,
    pub title: String,
    pub slug: String,
    pub content: Option<String>,
    pub category_id: Option<i64>,
    pub source_url: Option<String>,
    pub source_type: Option<String>,
    pub color: Option<String>,
    pub sort_order: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Tag {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Attachment {
    pub id: i64,
    pub article_id: i64,
    pub filename: String,
    pub original_name: String,
    pub file_type: String,
    pub file_size: i64,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ArticleWithTags {
    pub article: Article,
    pub tags: Vec<String>,
    pub attachments: Vec<Attachment>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TreeNode {
    pub id: i64,
    pub name: String,
    pub icon: Option<String>,
    pub children: Vec<TreeNode>,
    pub articles: Vec<ArticleSummary>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ArticleSummary {
    pub id: i64,
    pub title: String,
    pub slug: String,
    pub color: Option<String>,
    pub sort_order: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: i64,
    pub title: String,
    pub slug: String,
    pub snippet: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NewArticle {
    pub title: String,
    pub content: Option<String>,
    pub category_id: Option<i64>,
    pub source_url: Option<String>,
    pub source_type: Option<String>,
    pub color: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateArticle {
    pub id: i64,
    pub title: String,
    pub content: Option<String>,
    pub category_id: Option<i64>,
    pub source_url: Option<String>,
    pub source_type: Option<String>,
    pub color: Option<String>,
    pub tags: Vec<String>,
}

pub struct Database {
    pub conn: Mutex<Connection>,
}

impl Database {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open(path)?;
        // Enable WAL mode for better concurrent performance
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")?;
        let db = Database {
            conn: Mutex::new(conn),
        };
        db.init_schema()?;
        Ok(db)
    }

    fn init_schema(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute_batch(
            r#"
            -- Hierarchical categories (tree structure)
            CREATE TABLE IF NOT EXISTS categories (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                parent_id INTEGER REFERENCES categories(id),
                icon TEXT,
                sort_order INTEGER DEFAULT 0,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );

            -- Articles/entries
            CREATE TABLE IF NOT EXISTS articles (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                slug TEXT UNIQUE NOT NULL,
                content TEXT,
                category_id INTEGER REFERENCES categories(id),
                source_url TEXT,
                source_type TEXT,
                color TEXT,
                sort_order INTEGER DEFAULT 0,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );

            -- Tags for flexible organization
            CREATE TABLE IF NOT EXISTS tags (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT UNIQUE NOT NULL
            );

            CREATE TABLE IF NOT EXISTS article_tags (
                article_id INTEGER REFERENCES articles(id) ON DELETE CASCADE,
                tag_id INTEGER REFERENCES tags(id) ON DELETE CASCADE,
                PRIMARY KEY (article_id, tag_id)
            );

            -- Attachments
            CREATE TABLE IF NOT EXISTS attachments (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                article_id INTEGER NOT NULL REFERENCES articles(id) ON DELETE CASCADE,
                filename TEXT NOT NULL,
                original_name TEXT NOT NULL,
                file_type TEXT NOT NULL,
                file_size INTEGER DEFAULT 0,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );

            -- Full-text search
            CREATE VIRTUAL TABLE IF NOT EXISTS articles_fts USING fts5(
                title,
                content,
                content=articles,
                content_rowid=id
            );

            -- Triggers for FTS sync
            CREATE TRIGGER IF NOT EXISTS articles_ai AFTER INSERT ON articles BEGIN
                INSERT INTO articles_fts(rowid, title, content) VALUES (new.id, new.title, new.content);
            END;

            CREATE TRIGGER IF NOT EXISTS articles_ad AFTER DELETE ON articles BEGIN
                INSERT INTO articles_fts(articles_fts, rowid, title, content) VALUES('delete', old.id, old.title, old.content);
            END;

            CREATE TRIGGER IF NOT EXISTS articles_au AFTER UPDATE ON articles BEGIN
                INSERT INTO articles_fts(articles_fts, rowid, title, content) VALUES('delete', old.id, old.title, old.content);
                INSERT INTO articles_fts(rowid, title, content) VALUES (new.id, new.title, new.content);
            END;

            -- Create indexes
            CREATE INDEX IF NOT EXISTS idx_articles_category ON articles(category_id);
            CREATE INDEX IF NOT EXISTS idx_articles_slug ON articles(slug);
            CREATE INDEX IF NOT EXISTS idx_categories_parent ON categories(parent_id);
            CREATE INDEX IF NOT EXISTS idx_attachments_article ON attachments(article_id);
            "#,
        )?;

        // Add columns if they don't exist (for migration)
        let _ = conn.execute("ALTER TABLE articles ADD COLUMN color TEXT", []);
        let _ = conn.execute(
            "ALTER TABLE articles ADD COLUMN sort_order INTEGER DEFAULT 0",
            [],
        );

        // Insert root category if not exists
        conn.execute(
            "INSERT OR IGNORE INTO categories (id, name, parent_id, icon, sort_order) VALUES (1, 'Root', NULL, 'folder', 0)",
            [],
        )?;

        Ok(())
    }

    // Category operations
    pub fn get_categories(&self) -> Result<Vec<Category>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, parent_id, icon, sort_order, created_at FROM categories ORDER BY sort_order, name"
        )?;

        let categories = stmt
            .query_map([], |row| {
                Ok(Category {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    parent_id: row.get(2)?,
                    icon: row.get(3)?,
                    sort_order: row.get(4)?,
                    created_at: row.get(5)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        Ok(categories)
    }

    pub fn create_category(
        &self,
        name: &str,
        parent_id: Option<i64>,
        icon: Option<&str>,
    ) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        // Get max sort_order for siblings
        let max_order: i32 = conn
            .query_row(
                "SELECT COALESCE(MAX(sort_order), -1) FROM categories WHERE parent_id IS ?1",
                params![parent_id],
                |row| row.get(0),
            )
            .unwrap_or(-1);

        conn.execute(
            "INSERT INTO categories (name, parent_id, icon, sort_order) VALUES (?1, ?2, ?3, ?4)",
            params![name, parent_id, icon, max_order + 1],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn update_category(&self, id: i64, name: &str, icon: Option<&str>) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE categories SET name = ?1, icon = ?2 WHERE id = ?3",
            params![name, icon, id],
        )?;
        Ok(())
    }

    pub fn move_category(&self, id: i64, new_parent_id: Option<i64>) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        // Prevent circular reference: check if new_parent_id is a descendant of id
        if let Some(target_parent) = new_parent_id {
            // Can't move a category into itself
            if target_parent == id {
                return Err(rusqlite::Error::InvalidParameterName(
                    "Cannot move a category into itself".to_string(),
                ));
            }

            // Check if target is a descendant of the category being moved
            let mut current = Some(target_parent);
            while let Some(check_id) = current {
                let parent: Option<i64> = conn
                    .query_row(
                        "SELECT parent_id FROM categories WHERE id = ?1",
                        params![check_id],
                        |row| row.get(0),
                    )
                    .ok()
                    .flatten();

                if parent == Some(id) {
                    return Err(rusqlite::Error::InvalidParameterName(
                        "Cannot move a category into its own descendant".to_string(),
                    ));
                }
                current = parent;
            }
        }

        // Get max sort_order for new siblings
        let max_order: i32 = conn
            .query_row(
                "SELECT COALESCE(MAX(sort_order), -1) FROM categories WHERE parent_id IS ?1",
                params![new_parent_id],
                |row| row.get(0),
            )
            .unwrap_or(-1);

        conn.execute(
            "UPDATE categories SET parent_id = ?1, sort_order = ?2 WHERE id = ?3",
            params![new_parent_id, max_order + 1, id],
        )?;
        Ok(())
    }

    pub fn reorder_category(&self, id: i64, new_sort_order: i32) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        // Get current category info
        let (current_order, parent_id): (i32, Option<i64>) = conn.query_row(
            "SELECT sort_order, parent_id FROM categories WHERE id = ?1",
            params![id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;

        if current_order == new_sort_order {
            return Ok(());
        }

        // Shift other categories to make room
        if new_sort_order < current_order {
            // Moving up: shift items between new and old position down
            conn.execute(
                "UPDATE categories SET sort_order = sort_order + 1 WHERE parent_id IS ?1 AND sort_order >= ?2 AND sort_order < ?3 AND id != ?4",
                params![parent_id, new_sort_order, current_order, id],
            )?;
        } else {
            // Moving down: shift items between old and new position up
            conn.execute(
                "UPDATE categories SET sort_order = sort_order - 1 WHERE parent_id IS ?1 AND sort_order > ?2 AND sort_order <= ?3 AND id != ?4",
                params![parent_id, current_order, new_sort_order, id],
            )?;
        }

        // Set the new sort order
        conn.execute(
            "UPDATE categories SET sort_order = ?1 WHERE id = ?2",
            params![new_sort_order, id],
        )?;
        Ok(())
    }

    pub fn delete_category(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        // Move articles to uncategorized (NULL)
        conn.execute(
            "UPDATE articles SET category_id = NULL WHERE category_id = ?1",
            params![id],
        )?;
        // Move child categories to parent
        let parent_id: Option<i64> = conn
            .query_row(
                "SELECT parent_id FROM categories WHERE id = ?1",
                params![id],
                |row| row.get(0),
            )
            .ok()
            .flatten();

        conn.execute(
            "UPDATE categories SET parent_id = ?1 WHERE parent_id = ?2",
            params![parent_id, id],
        )?;
        conn.execute("DELETE FROM categories WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn get_tree(&self) -> Result<Vec<TreeNode>> {
        let categories = self.get_categories()?;
        let articles = self.get_all_article_summaries()?;

        fn build_tree(
            categories: &[Category],
            articles: &[(i64, String, String, Option<String>, i32, Option<i64>)],
            parent_id: Option<i64>,
        ) -> Vec<TreeNode> {
            let mut nodes: Vec<TreeNode> = categories
                .iter()
                .filter(|c| c.parent_id == parent_id)
                .map(|c| {
                    let mut cat_articles: Vec<ArticleSummary> = articles
                        .iter()
                        .filter(|a| a.5 == Some(c.id))
                        .map(|a| ArticleSummary {
                            id: a.0,
                            title: a.1.clone(),
                            slug: a.2.clone(),
                            color: a.3.clone(),
                            sort_order: a.4,
                        })
                        .collect();
                    cat_articles.sort_by_key(|a| a.sort_order);

                    TreeNode {
                        id: c.id,
                        name: c.name.clone(),
                        icon: c.icon.clone(),
                        children: build_tree(categories, articles, Some(c.id)),
                        articles: cat_articles,
                    }
                })
                .collect();
            nodes.sort_by_key(|n| {
                categories
                    .iter()
                    .find(|c| c.id == n.id)
                    .map(|c| c.sort_order)
                    .unwrap_or(0)
            });
            nodes
        }

        Ok(build_tree(&categories, &articles, None))
    }

    // Article operations
    fn get_all_article_summaries(
        &self,
    ) -> Result<Vec<(i64, String, String, Option<String>, i32, Option<i64>)>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, title, slug, color, sort_order, category_id FROM articles ORDER BY sort_order, title"
        )?;

        let articles = stmt
            .query_map([], |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                ))
            })?
            .collect::<Result<Vec<_>>>()?;

        Ok(articles)
    }

    pub fn get_article(&self, id: i64) -> Result<Option<ArticleWithTags>> {
        let conn = self.conn.lock().unwrap();
        let article = conn.query_row(
            "SELECT id, title, slug, content, category_id, source_url, source_type, color, sort_order, created_at, updated_at FROM articles WHERE id = ?1",
            params![id],
            |row| {
                Ok(Article {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    slug: row.get(2)?,
                    content: row.get(3)?,
                    category_id: row.get(4)?,
                    source_url: row.get(5)?,
                    source_type: row.get(6)?,
                    color: row.get(7)?,
                    sort_order: row.get(8)?,
                    created_at: row.get(9)?,
                    updated_at: row.get(10)?,
                })
            },
        );

        match article {
            Ok(article) => {
                let mut stmt = conn.prepare(
                    "SELECT t.name FROM tags t JOIN article_tags at ON t.id = at.tag_id WHERE at.article_id = ?1"
                )?;
                let tags = stmt
                    .query_map(params![article.id], |row| row.get(0))?
                    .collect::<Result<Vec<String>>>()?;

                let mut stmt = conn.prepare(
                    "SELECT id, article_id, filename, original_name, file_type, file_size, created_at FROM attachments WHERE article_id = ?1"
                )?;
                let attachments = stmt
                    .query_map(params![article.id], |row| {
                        Ok(Attachment {
                            id: row.get(0)?,
                            article_id: row.get(1)?,
                            filename: row.get(2)?,
                            original_name: row.get(3)?,
                            file_type: row.get(4)?,
                            file_size: row.get(5)?,
                            created_at: row.get(6)?,
                        })
                    })?
                    .collect::<Result<Vec<_>>>()?;

                Ok(Some(ArticleWithTags {
                    article,
                    tags,
                    attachments,
                }))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub fn get_article_by_slug(&self, slug: &str) -> Result<Option<ArticleWithTags>> {
        let conn = self.conn.lock().unwrap();
        let article_id: Option<i64> = conn
            .query_row(
                "SELECT id FROM articles WHERE slug = ?1",
                params![slug],
                |row| row.get(0),
            )
            .ok();

        drop(conn);

        match article_id {
            Some(id) => self.get_article(id),
            None => Ok(None),
        }
    }

    pub fn create_article(&self, article: &NewArticle) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let slug = slug::slugify(&article.title);

        // Get max sort_order for category
        let max_order: i32 = conn
            .query_row(
                "SELECT COALESCE(MAX(sort_order), -1) FROM articles WHERE category_id IS ?1",
                params![article.category_id],
                |row| row.get(0),
            )
            .unwrap_or(-1);

        conn.execute(
            "INSERT INTO articles (title, slug, content, category_id, source_url, source_type, color, sort_order) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![article.title, slug, article.content, article.category_id, article.source_url, article.source_type, article.color, max_order + 1],
        )?;

        let article_id = conn.last_insert_rowid();

        // Add tags
        for tag_name in &article.tags {
            conn.execute(
                "INSERT OR IGNORE INTO tags (name) VALUES (?1)",
                params![tag_name],
            )?;
            conn.execute(
                "INSERT OR IGNORE INTO article_tags (article_id, tag_id) SELECT ?1, id FROM tags WHERE name = ?2",
                params![article_id, tag_name],
            )?;
        }

        Ok(article_id)
    }

    pub fn update_article(&self, article: &UpdateArticle) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let slug = slug::slugify(&article.title);

        conn.execute(
            "UPDATE articles SET title = ?1, slug = ?2, content = ?3, category_id = ?4, source_url = ?5, source_type = ?6, color = ?7, updated_at = CURRENT_TIMESTAMP WHERE id = ?8",
            params![article.title, slug, article.content, article.category_id, article.source_url, article.source_type, article.color, article.id],
        )?;

        // Update tags - remove old and add new
        conn.execute(
            "DELETE FROM article_tags WHERE article_id = ?1",
            params![article.id],
        )?;

        for tag_name in &article.tags {
            conn.execute(
                "INSERT OR IGNORE INTO tags (name) VALUES (?1)",
                params![tag_name],
            )?;
            conn.execute(
                "INSERT OR IGNORE INTO article_tags (article_id, tag_id) SELECT ?1, id FROM tags WHERE name = ?2",
                params![article.id, tag_name],
            )?;
        }

        Ok(())
    }

    pub fn delete_article(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "DELETE FROM article_tags WHERE article_id = ?1",
            params![id],
        )?;
        conn.execute("DELETE FROM attachments WHERE article_id = ?1", params![id])?;
        conn.execute("DELETE FROM articles WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn move_article(&self, id: i64, category_id: Option<i64>) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        // Get max sort_order for new category
        let max_order: i32 = conn
            .query_row(
                "SELECT COALESCE(MAX(sort_order), -1) FROM articles WHERE category_id IS ?1",
                params![category_id],
                |row| row.get(0),
            )
            .unwrap_or(-1);

        conn.execute(
            "UPDATE articles SET category_id = ?1, sort_order = ?2 WHERE id = ?3",
            params![category_id, max_order + 1, id],
        )?;
        Ok(())
    }

    pub fn reorder_article(&self, id: i64, new_sort_order: i32) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        // Get current article info
        let (current_order, category_id): (i32, Option<i64>) = conn.query_row(
            "SELECT sort_order, category_id FROM articles WHERE id = ?1",
            params![id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;

        if current_order == new_sort_order {
            return Ok(());
        }

        // Shift other articles to make room
        if new_sort_order < current_order {
            // Moving up: shift items between new and old position down
            conn.execute(
                "UPDATE articles SET sort_order = sort_order + 1 WHERE category_id IS ?1 AND sort_order >= ?2 AND sort_order < ?3 AND id != ?4",
                params![category_id, new_sort_order, current_order, id],
            )?;
        } else {
            // Moving down: shift items between old and new position up
            conn.execute(
                "UPDATE articles SET sort_order = sort_order - 1 WHERE category_id IS ?1 AND sort_order > ?2 AND sort_order <= ?3 AND id != ?4",
                params![category_id, current_order, new_sort_order, id],
            )?;
        }

        // Set the new sort order
        conn.execute(
            "UPDATE articles SET sort_order = ?1 WHERE id = ?2",
            params![new_sort_order, id],
        )?;
        Ok(())
    }

    // Attachment operations
    pub fn add_attachment(
        &self,
        article_id: i64,
        filename: &str,
        original_name: &str,
        file_type: &str,
        file_size: i64,
    ) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO attachments (article_id, filename, original_name, file_type, file_size) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![article_id, filename, original_name, file_type, file_size],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn get_attachments(&self, article_id: i64) -> Result<Vec<Attachment>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, article_id, filename, original_name, file_type, file_size, created_at FROM attachments WHERE article_id = ?1"
        )?;

        let attachments = stmt
            .query_map(params![article_id], |row| {
                Ok(Attachment {
                    id: row.get(0)?,
                    article_id: row.get(1)?,
                    filename: row.get(2)?,
                    original_name: row.get(3)?,
                    file_type: row.get(4)?,
                    file_size: row.get(5)?,
                    created_at: row.get(6)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        Ok(attachments)
    }

    pub fn delete_attachment(&self, id: i64) -> Result<String> {
        let conn = self.conn.lock().unwrap();
        let filename: String = conn.query_row(
            "SELECT filename FROM attachments WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )?;
        conn.execute("DELETE FROM attachments WHERE id = ?1", params![id])?;
        Ok(filename)
    }

    // Search
    pub fn search(&self, query: &str) -> Result<Vec<SearchResult>> {
        let conn = self.conn.lock().unwrap();
        let search_query = format!("{}*", query);

        let mut stmt = conn.prepare(
            r#"
            SELECT a.id, a.title, a.slug, snippet(articles_fts, 1, '<mark>', '</mark>', '...', 32) as snippet
            FROM articles_fts
            JOIN articles a ON a.id = articles_fts.rowid
            WHERE articles_fts MATCH ?1
            ORDER BY rank
            LIMIT 50
            "#
        )?;

        let results = stmt
            .query_map(params![search_query], |row| {
                Ok(SearchResult {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    slug: row.get(2)?,
                    snippet: row.get(3)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        Ok(results)
    }

    // Tag operations
    pub fn get_all_tags(&self) -> Result<Vec<Tag>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, name FROM tags ORDER BY name")?;

        let tags = stmt
            .query_map([], |row| {
                Ok(Tag {
                    id: row.get(0)?,
                    name: row.get(1)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        Ok(tags)
    }

    pub fn get_articles_by_tag(&self, tag_name: &str) -> Result<Vec<ArticleSummary>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"
            SELECT a.id, a.title, a.slug, a.color, a.sort_order
            FROM articles a
            JOIN article_tags at ON a.id = at.article_id
            JOIN tags t ON t.id = at.tag_id
            WHERE t.name = ?1
            ORDER BY a.title
            "#,
        )?;

        let articles = stmt
            .query_map(params![tag_name], |row| {
                Ok(ArticleSummary {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    slug: row.get(2)?,
                    color: row.get(3)?,
                    sort_order: row.get(4)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        Ok(articles)
    }
}
