use rusqlite::ffi::sqlite3_auto_extension;

#[test]
fn test_baize() -> anyhow::Result<()> {
    let db = init()?;

    db.execute(
        "CREATE VIRTUAL TABLE test USING fts5(content, tokenize='baize')",
        [],
    )?;

    db.execute(
        "INSERT INTO test(content) VALUES ('1！5！ 哥们在这给你说唱')",
        [],
    )?;
    let count: i64 = db.query_row("SELECT count(*) FROM test", [], |row| row.get(0))?;
    assert_eq!(count, 1);

    db.query_row("SELECT 1 FROM test WHERE test MATCH '哥们'", [], |row| {
        let data: i64 = row.get(0)?;
        assert_eq!(data, 1);
        Ok(())
    })?;

    db.query_row("SELECT 1 FROM test WHERE test MATCH '说唱'", [], |row| {
        let data: i64 = row.get(0)?;
        assert_eq!(data, 1);
        Ok(())
    })?;

    let count: i64 = db.query_row(
        "SELECT count(*) FROM test WHERE test MATCH '15'",
        [],
        |row| row.get(0),
    )?;
    assert_eq!(count, 0);

    Ok(())
}

#[test]
fn test_jieba() -> anyhow::Result<()> {
    let db = init()?;

    db.execute(
        "CREATE VIRTUAL TABLE test USING fts5(content, tokenize='jieba')",
        [],
    )?;

    db.execute(
        "INSERT INTO test(content) VALUES ('我来到北京清华大学')",
        [],
    )?;

    db.query_row("SELECT 1 FROM test WHERE test MATCH '北京'", [], |row| {
        let data: i64 = row.get(0)?;
        assert_eq!(data, 1);
        Ok(())
    })?;
    db.query_row("SELECT 1 FROM test WHERE test MATCH '清华'", [], |row| {
        let data: i64 = row.get(0)?;
        assert_eq!(data, 1);
        Ok(())
    })?;
    db.query_row("SELECT 1 FROM test WHERE test MATCH '清华大学'", [], |row| {
        let data: i64 = row.get(0)?;
        assert_eq!(data, 1);
        Ok(())
    })?;


    Ok(())
}

fn init() -> anyhow::Result<rusqlite::Connection> {
    unsafe {
        sqlite3_auto_extension(Some(std::mem::transmute(
            crate::sqlite3_extension_init as *const (),
        )));
    }

    Ok(rusqlite::Connection::open_in_memory()?)
}
