use crate::db::{
    ArticleSummary, ArticleWithTags, Attachment, Database, NewArticle, SearchResult, Tag, TreeNode,
    UpdateArticle,
};
use std::fs;
use std::path::PathBuf;
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub fn get_tree(db: State<'_, Database>) -> Result<Vec<TreeNode>, String> {
    db.get_tree().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_article(db: State<'_, Database>, id: i64) -> Result<Option<ArticleWithTags>, String> {
    db.get_article(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_article_by_slug(
    db: State<'_, Database>,
    slug: String,
) -> Result<Option<ArticleWithTags>, String> {
    db.get_article_by_slug(&slug).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn search(db: State<'_, Database>, query: String) -> Result<Vec<SearchResult>, String> {
    db.search(&query).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_article(
    db: State<'_, Database>,
    title: String,
    content: Option<String>,
    category_id: Option<i64>,
    source_url: Option<String>,
    source_type: Option<String>,
    color: Option<String>,
    tags: Vec<String>,
) -> Result<i64, String> {
    let article = NewArticle {
        title,
        content,
        category_id,
        source_url,
        source_type,
        color,
        tags,
    };
    db.create_article(&article).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_article(
    db: State<'_, Database>,
    id: i64,
    title: String,
    content: Option<String>,
    category_id: Option<i64>,
    source_url: Option<String>,
    source_type: Option<String>,
    color: Option<String>,
    tags: Vec<String>,
) -> Result<(), String> {
    let article = UpdateArticle {
        id,
        title,
        content,
        category_id,
        source_url,
        source_type,
        color,
        tags,
    };
    db.update_article(&article).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_article(db: State<'_, Database>, id: i64) -> Result<(), String> {
    db.delete_article(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn move_article(
    db: State<'_, Database>,
    id: i64,
    category_id: Option<i64>,
) -> Result<(), String> {
    db.move_article(id, category_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn reorder_article(
    db: State<'_, Database>,
    id: i64,
    new_sort_order: i32,
) -> Result<(), String> {
    db.reorder_article(id, new_sort_order)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_category(
    db: State<'_, Database>,
    name: String,
    parent_id: Option<i64>,
    icon: Option<String>,
) -> Result<i64, String> {
    db.create_category(&name, parent_id, icon.as_deref())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_category(
    db: State<'_, Database>,
    id: i64,
    name: String,
    icon: Option<String>,
) -> Result<(), String> {
    db.update_category(id, &name, icon.as_deref())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn move_category(
    db: State<'_, Database>,
    id: i64,
    new_parent_id: Option<i64>,
) -> Result<(), String> {
    db.move_category(id, new_parent_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn reorder_category(
    db: State<'_, Database>,
    id: i64,
    new_sort_order: i32,
) -> Result<(), String> {
    db.reorder_category(id, new_sort_order)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_category(db: State<'_, Database>, id: i64) -> Result<(), String> {
    db.delete_category(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_all_tags(db: State<'_, Database>) -> Result<Vec<Tag>, String> {
    db.get_all_tags().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_articles_by_tag(
    db: State<'_, Database>,
    tag_name: String,
) -> Result<Vec<ArticleSummary>, String> {
    db.get_articles_by_tag(&tag_name).map_err(|e| e.to_string())
}

// Attachment commands
#[tauri::command]
pub fn add_attachment(
    db: State<'_, Database>,
    attachments_dir: State<'_, PathBuf>,
    article_id: i64,
    source_path: String,
    original_name: String,
) -> Result<Attachment, String> {
    // Generate unique filename
    let extension = std::path::Path::new(&original_name)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    let unique_name = format!("{}_{}.{}", article_id, Uuid::new_v4(), extension);
    let dest_path = attachments_dir.join(&unique_name);

    // Copy file to attachments directory
    fs::copy(&source_path, &dest_path).map_err(|e| format!("Failed to copy file: {}", e))?;

    // Get file size and type
    let metadata =
        fs::metadata(&dest_path).map_err(|e| format!("Failed to get file metadata: {}", e))?;
    let file_size = metadata.len() as i64;
    let file_type = mime_guess::from_path(&original_name)
        .first_or_octet_stream()
        .to_string();

    // Add to database
    let id = db
        .add_attachment(
            article_id,
            &unique_name,
            &original_name,
            &file_type,
            file_size,
        )
        .map_err(|e| e.to_string())?;

    Ok(Attachment {
        id,
        article_id,
        filename: unique_name,
        original_name,
        file_type,
        file_size,
        created_at: chrono::Utc::now().to_rfc3339(),
    })
}

#[tauri::command]
pub fn get_attachments(
    db: State<'_, Database>,
    article_id: i64,
) -> Result<Vec<Attachment>, String> {
    db.get_attachments(article_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_attachment(
    db: State<'_, Database>,
    attachments_dir: State<'_, PathBuf>,
    id: i64,
) -> Result<(), String> {
    // Get filename and delete from database
    let filename = db.delete_attachment(id).map_err(|e| e.to_string())?;

    // Delete file from filesystem
    let file_path = attachments_dir.join(&filename);
    if file_path.exists() {
        fs::remove_file(file_path).map_err(|e| format!("Failed to delete file: {}", e))?;
    }

    Ok(())
}
