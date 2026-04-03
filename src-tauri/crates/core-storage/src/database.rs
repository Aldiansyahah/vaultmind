use rusqlite::{Connection, OptionalExtension};

use crate::error::Result;
use crate::models::{Note, Tag};

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new(conn: Connection) -> Result<Self> {
        let db = Self { conn };
        crate::migrations::init_database(&db.conn)?;
        Ok(db)
    }

    pub fn conn(&self) -> &Connection {
        &self.conn
    }

    // Note CRUD operations
    pub fn create_note(&self, path: &str, title: &str, content_hash: &str) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO notes (path, title, content_hash) VALUES (?1, ?2, ?3)",
            (path, title, content_hash),
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_note_by_id(&self, id: i64) -> Result<Option<Note>> {
        let note = self
            .conn
            .query_row(
                "SELECT id, path, title, content_hash, created_at, updated_at FROM notes WHERE id = ?1",
                [id],
                |row| {
                    Ok(Note {
                        id: Some(row.get(0)?),
                        path: row.get(1)?,
                        title: row.get(2)?,
                        content_hash: row.get(3)?,
                        created_at: row.get(4)?,
                        updated_at: row.get(5)?,
                    })
                },
            )
            .optional()?;
        Ok(note)
    }

    pub fn get_note_by_path(&self, path: &str) -> Result<Option<Note>> {
        let note = self
            .conn
            .query_row(
                "SELECT id, path, title, content_hash, created_at, updated_at FROM notes WHERE path = ?1",
                [path],
                |row| {
                    Ok(Note {
                        id: Some(row.get(0)?),
                        path: row.get(1)?,
                        title: row.get(2)?,
                        content_hash: row.get(3)?,
                        created_at: row.get(4)?,
                        updated_at: row.get(5)?,
                    })
                },
            )
            .optional()?;
        Ok(note)
    }

    pub fn update_note(&self, id: i64, title: &str, content_hash: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE notes SET title = ?1, content_hash = ?2, updated_at = datetime('now') WHERE id = ?3",
            (title, content_hash, id),
        )?;
        Ok(())
    }

    pub fn delete_note(&self, id: i64) -> Result<()> {
        self.conn.execute("DELETE FROM notes WHERE id = ?1", [id])?;
        Ok(())
    }

    pub fn list_notes(&self) -> Result<Vec<Note>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, path, title, content_hash, created_at, updated_at FROM notes ORDER BY updated_at DESC"
        )?;
        let notes = stmt
            .query_map([], |row| {
                Ok(Note {
                    id: Some(row.get(0)?),
                    path: row.get(1)?,
                    title: row.get(2)?,
                    content_hash: row.get(3)?,
                    created_at: row.get(4)?,
                    updated_at: row.get(5)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(notes)
    }

    // Tag CRUD operations
    pub fn create_tag(&self, name: &str) -> Result<i64> {
        self.conn
            .execute("INSERT INTO tags (name) VALUES (?1)", [name])?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_tag_by_id(&self, id: i64) -> Result<Option<Tag>> {
        let tag = self
            .conn
            .query_row("SELECT id, name FROM tags WHERE id = ?1", [id], |row| {
                Ok(Tag {
                    id: Some(row.get(0)?),
                    name: row.get(1)?,
                })
            })
            .optional()?;
        Ok(tag)
    }

    pub fn get_tag_by_name(&self, name: &str) -> Result<Option<Tag>> {
        let tag = self
            .conn
            .query_row("SELECT id, name FROM tags WHERE name = ?1", [name], |row| {
                Ok(Tag {
                    id: Some(row.get(0)?),
                    name: row.get(1)?,
                })
            })
            .optional()?;
        Ok(tag)
    }

    pub fn delete_tag(&self, id: i64) -> Result<()> {
        self.conn.execute("DELETE FROM tags WHERE id = ?1", [id])?;
        Ok(())
    }

    pub fn list_tags(&self) -> Result<Vec<Tag>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, name FROM tags ORDER BY name")?;
        let tags = stmt
            .query_map([], |row| {
                Ok(Tag {
                    id: Some(row.get(0)?),
                    name: row.get(1)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(tags)
    }

    // Note-Tag association operations
    pub fn add_tag_to_note(&self, note_id: i64, tag_id: i64) -> Result<()> {
        self.conn.execute(
            "INSERT OR IGNORE INTO note_tags (note_id, tag_id) VALUES (?1, ?2)",
            (note_id, tag_id),
        )?;
        Ok(())
    }

    pub fn remove_tag_from_note(&self, note_id: i64, tag_id: i64) -> Result<()> {
        self.conn.execute(
            "DELETE FROM note_tags WHERE note_id = ?1 AND tag_id = ?2",
            (note_id, tag_id),
        )?;
        Ok(())
    }

    pub fn get_tags_for_note(&self, note_id: i64) -> Result<Vec<Tag>> {
        let mut stmt = self.conn.prepare(
            "SELECT t.id, t.name FROM tags t INNER JOIN note_tags nt ON t.id = nt.tag_id WHERE nt.note_id = ?1 ORDER BY t.name"
        )?;
        let tags = stmt
            .query_map([note_id], |row| {
                Ok(Tag {
                    id: Some(row.get(0)?),
                    name: row.get(1)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(tags)
    }

    pub fn get_notes_for_tag(&self, tag_id: i64) -> Result<Vec<Note>> {
        let mut stmt = self.conn.prepare(
            "SELECT n.id, n.path, n.title, n.content_hash, n.created_at, n.updated_at FROM notes n INNER JOIN note_tags nt ON n.id = nt.note_id WHERE nt.tag_id = ?1 ORDER BY n.updated_at DESC"
        )?;
        let notes = stmt
            .query_map([tag_id], |row| {
                Ok(Note {
                    id: Some(row.get(0)?),
                    path: row.get(1)?,
                    title: row.get(2)?,
                    content_hash: row.get(3)?,
                    created_at: row.get(4)?,
                    updated_at: row.get(5)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(notes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::hash::{DefaultHasher, Hash, Hasher};

    fn create_test_db() -> Database {
        let conn = Connection::open_in_memory().expect("Failed to create in-memory database");
        Database::new(conn).expect("Failed to initialize database")
    }

    fn hash_content(content: &str) -> String {
        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    #[test]
    fn test_create_and_get_note() {
        let db = create_test_db();
        let hash = hash_content("test content");
        let id = db
            .create_note("test.md", "Test Note", &hash)
            .expect("Failed to create note");

        let note = db
            .get_note_by_id(id)
            .expect("Failed to get note")
            .expect("Note not found");
        assert_eq!(note.path, "test.md");
        assert_eq!(note.title, "Test Note");
        assert_eq!(note.content_hash, hash);
        assert!(note.created_at.len() > 0);
        assert!(note.updated_at.len() > 0);
    }

    #[test]
    fn test_get_note_by_path() {
        let db = create_test_db();
        let hash = hash_content("test content");
        db.create_note("notes/test.md", "Test", &hash)
            .expect("Failed to create note");

        let note = db
            .get_note_by_path("notes/test.md")
            .expect("Failed to get note")
            .expect("Note not found");
        assert_eq!(note.path, "notes/test.md");
    }

    #[test]
    fn test_get_nonexistent_note() {
        let db = create_test_db();
        let note = db.get_note_by_id(999).expect("Failed to query note");
        assert!(note.is_none());
    }

    #[test]
    fn test_update_note() {
        let db = create_test_db();
        let hash = hash_content("original");
        let id = db
            .create_note("test.md", "Original", &hash)
            .expect("Failed to create note");

        let new_hash = hash_content("updated");
        db.update_note(id, "Updated Title", &new_hash)
            .expect("Failed to update note");

        let note = db
            .get_note_by_id(id)
            .expect("Failed to get note")
            .expect("Note not found");
        assert_eq!(note.title, "Updated Title");
        assert_eq!(note.content_hash, new_hash);
    }

    #[test]
    fn test_delete_note() {
        let db = create_test_db();
        let hash = hash_content("test");
        let id = db
            .create_note("test.md", "Test", &hash)
            .expect("Failed to create note");

        db.delete_note(id).expect("Failed to delete note");
        let note = db.get_note_by_id(id).expect("Failed to query note");
        assert!(note.is_none());
    }

    #[test]
    fn test_list_notes() {
        let db = create_test_db();
        let hash = hash_content("content");
        db.create_note("a.md", "Note A", &hash)
            .expect("Failed to create note");
        db.create_note("b.md", "Note B", &hash)
            .expect("Failed to create note");

        let notes = db.list_notes().expect("Failed to list notes");
        assert_eq!(notes.len(), 2);
    }

    #[test]
    fn test_create_and_get_tag() {
        let db = create_test_db();
        let id = db.create_tag("rust").expect("Failed to create tag");

        let tag = db
            .get_tag_by_id(id)
            .expect("Failed to get tag")
            .expect("Tag not found");
        assert_eq!(tag.name, "rust");
    }

    #[test]
    fn test_get_tag_by_name() {
        let db = create_test_db();
        db.create_tag("programming").expect("Failed to create tag");

        let tag = db
            .get_tag_by_name("programming")
            .expect("Failed to get tag")
            .expect("Tag not found");
        assert_eq!(tag.name, "programming");
    }

    #[test]
    fn test_delete_tag() {
        let db = create_test_db();
        let id = db.create_tag("temp").expect("Failed to create tag");

        db.delete_tag(id).expect("Failed to delete tag");
        let tag = db.get_tag_by_id(id).expect("Failed to query tag");
        assert!(tag.is_none());
    }

    #[test]
    fn test_list_tags() {
        let db = create_test_db();
        db.create_tag("alpha").expect("Failed to create tag");
        db.create_tag("beta").expect("Failed to create tag");

        let tags = db.list_tags().expect("Failed to list tags");
        assert_eq!(tags.len(), 2);
        assert_eq!(tags[0].name, "alpha");
        assert_eq!(tags[1].name, "beta");
    }

    #[test]
    fn test_add_tag_to_note() {
        let db = create_test_db();
        let hash = hash_content("content");
        let note_id = db
            .create_note("test.md", "Test", &hash)
            .expect("Failed to create note");
        let tag_id = db.create_tag("test-tag").expect("Failed to create tag");

        db.add_tag_to_note(note_id, tag_id)
            .expect("Failed to add tag");

        let tags = db.get_tags_for_note(note_id).expect("Failed to get tags");
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].name, "test-tag");
    }

    #[test]
    fn test_remove_tag_from_note() {
        let db = create_test_db();
        let hash = hash_content("content");
        let note_id = db
            .create_note("test.md", "Test", &hash)
            .expect("Failed to create note");
        let tag_id = db.create_tag("test-tag").expect("Failed to create tag");

        db.add_tag_to_note(note_id, tag_id)
            .expect("Failed to add tag");
        db.remove_tag_from_note(note_id, tag_id)
            .expect("Failed to remove tag");

        let tags = db.get_tags_for_note(note_id).expect("Failed to get tags");
        assert_eq!(tags.len(), 0);
    }

    #[test]
    fn test_get_notes_for_tag() {
        let db = create_test_db();
        let hash = hash_content("content");
        let note_id1 = db
            .create_note("a.md", "Note A", &hash)
            .expect("Failed to create note");
        let note_id2 = db
            .create_note("b.md", "Note B", &hash)
            .expect("Failed to create note");
        let tag_id = db.create_tag("shared-tag").expect("Failed to create tag");

        db.add_tag_to_note(note_id1, tag_id)
            .expect("Failed to add tag");
        db.add_tag_to_note(note_id2, tag_id)
            .expect("Failed to add tag");

        let notes = db.get_notes_for_tag(tag_id).expect("Failed to get notes");
        assert_eq!(notes.len(), 2);
    }

    #[test]
    fn test_cascade_delete_note_removes_note_tags() {
        let db = create_test_db();
        let hash = hash_content("content");
        let note_id = db
            .create_note("test.md", "Test", &hash)
            .expect("Failed to create note");
        let tag_id = db.create_tag("test-tag").expect("Failed to create tag");

        db.add_tag_to_note(note_id, tag_id)
            .expect("Failed to add tag");
        db.delete_note(note_id).expect("Failed to delete note");

        let notes = db.get_notes_for_tag(tag_id).expect("Failed to get notes");
        assert_eq!(notes.len(), 0);
    }

    #[test]
    fn test_duplicate_path_rejected() {
        let db = create_test_db();
        let hash = hash_content("content");
        db.create_note("test.md", "Test", &hash)
            .expect("Failed to create note");

        let result = db.create_note("test.md", "Duplicate", &hash);
        assert!(result.is_err());
    }

    #[test]
    fn test_add_duplicate_tag_to_note_is_ignored() {
        let db = create_test_db();
        let hash = hash_content("content");
        let note_id = db
            .create_note("test.md", "Test", &hash)
            .expect("Failed to create note");
        let tag_id = db.create_tag("test-tag").expect("Failed to create tag");

        db.add_tag_to_note(note_id, tag_id)
            .expect("Failed to add tag");
        db.add_tag_to_note(note_id, tag_id)
            .expect("Failed to add tag again");

        let tags = db.get_tags_for_note(note_id).expect("Failed to get tags");
        assert_eq!(tags.len(), 1);
    }
}
