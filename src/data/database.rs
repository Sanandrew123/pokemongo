// 数据库系统
// 开发心理：数据库提供持久化存储、事务支持、查询优化和数据完整性保证
// 设计原则：ACID特性、查询优化、连接池管理、备份恢复

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use log::{debug, warn, error, info};
use crate::core::error::GameError;
use super::DataType;

// 游戏数据库
pub struct GameDatabase {
    // 数据库连接
    connection: DatabaseConnection,
    
    // 配置
    config: DatabaseConfig,
    
    // 缓存
    query_cache: HashMap<String, QueryResult>,
    
    // 统计信息
    statistics: DatabaseStatistics,
    
    // 事务管理
    transaction_manager: TransactionManager,
}

// 数据库连接
pub enum DatabaseConnection {
    SQLite(rusqlite::Connection),
    // 可以扩展支持其他数据库
    // PostgreSQL(postgres::Client),
    // MySQL(mysql::Conn),
}

// 数据库配置
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub db_path: String,
    pub max_connections: u32,
    pub connection_timeout: std::time::Duration,
    pub query_timeout: std::time::Duration,
    pub auto_vacuum: bool,
    pub cache_size: i32,
    pub journal_mode: JournalMode,
    pub synchronous: SynchronousMode,
}

// 日志模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JournalMode {
    Delete,
    Truncate,
    Persist,
    Memory,
    WAL,
    Off,
}

// 同步模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SynchronousMode {
    Off,
    Normal,
    Full,
    Extra,
}

// 查询结果
#[derive(Debug, Clone)]
pub struct QueryResult {
    pub data: Vec<u8>,
    pub rows_affected: usize,
    pub execution_time: std::time::Duration,
    pub cached: bool,
}

// 数据库统计
#[derive(Debug, Clone, Default)]
pub struct DatabaseStatistics {
    pub queries_executed: u64,
    pub queries_cached: u64,
    pub transactions_committed: u64,
    pub transactions_rolled_back: u64,
    pub total_query_time: std::time::Duration,
    pub average_query_time: std::time::Duration,
    pub cache_hit_rate: f32,
}

// 事务管理器
pub struct TransactionManager {
    active_transactions: HashMap<String, Transaction>,
    transaction_counter: u64,
}

// 事务
#[derive(Debug, Clone)]
pub struct Transaction {
    pub id: String,
    pub started_at: std::time::Instant,
    pub isolation_level: IsolationLevel,
    pub read_only: bool,
    pub statements: Vec<String>,
}

// 隔离级别
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IsolationLevel {
    ReadUncommitted,
    ReadCommitted,
    RepeatableRead,
    Serializable,
}

// 查询构建器
pub struct QueryBuilder {
    query_type: QueryType,
    table: String,
    columns: Vec<String>,
    conditions: Vec<Condition>,
    joins: Vec<Join>,
    order_by: Vec<OrderBy>,
    group_by: Vec<String>,
    having: Vec<Condition>,
    limit: Option<u32>,
    offset: Option<u32>,
}

// 查询类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryType {
    Select,
    Insert,
    Update,
    Delete,
}

// 条件
#[derive(Debug, Clone)]
pub struct Condition {
    pub column: String,
    pub operator: ComparisonOperator,
    pub value: QueryValue,
    pub logical_op: Option<LogicalOperator>,
}

// 比较操作符
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComparisonOperator {
    Equal,
    NotEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    Like,
    In,
    NotIn,
    IsNull,
    IsNotNull,
}

// 逻辑操作符
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogicalOperator {
    And,
    Or,
}

// 连接
#[derive(Debug, Clone)]
pub struct Join {
    pub join_type: JoinType,
    pub table: String,
    pub on_condition: String,
}

// 连接类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JoinType {
    Inner,
    Left,
    Right,
    Full,
}

// 排序
#[derive(Debug, Clone)]
pub struct OrderBy {
    pub column: String,
    pub direction: SortDirection,
}

// 排序方向
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    Ascending,
    Descending,
}

// 查询值
#[derive(Debug, Clone)]
pub enum QueryValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Null,
    Array(Vec<QueryValue>),
}

impl GameDatabase {
    pub fn new(db_path: &str) -> Result<Self, GameError> {
        let config = DatabaseConfig::default(db_path);
        
        // 创建SQLite连接
        let connection = rusqlite::Connection::open(db_path)
            .map_err(|e| GameError::Database(format!("打开数据库失败: {}", e)))?;
        
        let mut database = Self {
            connection: DatabaseConnection::SQLite(connection),
            config,
            query_cache: HashMap::new(),
            statistics: DatabaseStatistics::default(),
            transaction_manager: TransactionManager::new(),
        };
        
        // 初始化数据库
        database.initialize()?;
        
        info!("数据库初始化完成: {}", db_path);
        Ok(database)
    }
    
    // 初始化数据库表结构
    pub fn initialize(&mut self) -> Result<(), GameError> {
        self.execute_sql("PRAGMA foreign_keys = ON")?;
        self.execute_sql(&format!("PRAGMA cache_size = {}", self.config.cache_size))?;
        self.execute_sql(&format!("PRAGMA journal_mode = {:?}", self.config.journal_mode))?;
        self.execute_sql(&format!("PRAGMA synchronous = {:?}", self.config.synchronous))?;
        
        // 创建数据表
        self.create_tables()?;
        
        debug!("数据库表结构初始化完成");
        Ok(())
    }
    
    // 加载数据
    pub fn load_data<T>(&mut self, data_type: DataType, id: &str) -> Result<T, GameError>
    where
        T: for<'de> Deserialize<'de>,
    {
        let table_name = self.get_table_name(data_type);
        let query = format!("SELECT data FROM {} WHERE id = ?", table_name);
        
        let data_bytes = self.query_single(&query, &[QueryValue::String(id.to_string())])?;
        
        // 反序列化数据
        let data: T = serde_json::from_slice(&data_bytes)
            .map_err(|e| GameError::Database(format!("反序列化失败: {}", e)))?;
        
        debug!("从数据库加载数据: {} ({})", table_name, id);
        Ok(data)
    }
    
    // 保存数据
    pub fn save_data<T>(&mut self, data_type: DataType, id: &str, data: &T) -> Result<(), GameError>
    where
        T: Serialize,
    {
        let table_name = self.get_table_name(data_type);
        
        // 序列化数据
        let data_bytes = serde_json::to_vec(data)
            .map_err(|e| GameError::Database(format!("序列化失败: {}", e)))?;
        
        let query = format!(
            "INSERT OR REPLACE INTO {} (id, data, created_at, updated_at) VALUES (?, ?, ?, ?)",
            table_name
        );
        
        let now = chrono::Utc::now().timestamp();
        let params = vec![
            QueryValue::String(id.to_string()),
            QueryValue::String(base64::encode(&data_bytes)),
            QueryValue::Integer(now),
            QueryValue::Integer(now),
        ];
        
        self.execute(&query, &params)?;
        
        debug!("保存数据到数据库: {} ({})", table_name, id);
        Ok(())
    }
    
    // 删除数据
    pub fn delete_data(&mut self, data_type: DataType, id: &str) -> Result<bool, GameError> {
        let table_name = self.get_table_name(data_type);
        let query = format!("DELETE FROM {} WHERE id = ?", table_name);
        
        let result = self.execute(&query, &[QueryValue::String(id.to_string())])?;
        let deleted = result.rows_affected > 0;
        
        debug!("删除数据: {} ({}) - 成功: {}", table_name, id, deleted);
        Ok(deleted)
    }
    
    // 批量加载数据
    pub fn load_data_batch<T>(&mut self, data_type: DataType, ids: &[String]) -> Result<HashMap<String, T>, GameError>
    where
        T: for<'de> Deserialize<'de>,
    {
        if ids.is_empty() {
            return Ok(HashMap::new());
        }
        
        let table_name = self.get_table_name(data_type);
        let placeholders: Vec<String> = ids.iter().map(|_| "?".to_string()).collect();
        let query = format!("SELECT id, data FROM {} WHERE id IN ({})", table_name, placeholders.join(","));
        
        let params: Vec<QueryValue> = ids.iter().map(|id| QueryValue::String(id.clone())).collect();
        let rows = self.query_multiple(&query, &params)?;
        
        let mut results = HashMap::new();
        
        for row in rows {
            if row.len() >= 2 {
                if let (QueryValue::String(id), QueryValue::String(data_str)) = (&row[0], &row[1]) {
                    let data_bytes = base64::decode(data_str)
                        .map_err(|e| GameError::Database(format!("Base64解码失败: {}", e)))?;
                    
                    let data: T = serde_json::from_slice(&data_bytes)
                        .map_err(|e| GameError::Database(format!("反序列化失败: {}", e)))?;
                    
                    results.insert(id.clone(), data);
                }
            }
        }
        
        debug!("批量加载数据: {} 请求: {}, 返回: {}", table_name, ids.len(), results.len());
        Ok(results)
    }
    
    // 开始事务
    pub fn begin_transaction(&mut self, isolation_level: Option<IsolationLevel>) -> Result<String, GameError> {
        let transaction_id = self.transaction_manager.begin_transaction(isolation_level.unwrap_or(IsolationLevel::ReadCommitted));
        
        self.execute_sql("BEGIN TRANSACTION")?;
        
        debug!("开始事务: {}", transaction_id);
        Ok(transaction_id)
    }
    
    // 提交事务
    pub fn commit_transaction(&mut self, transaction_id: &str) -> Result<(), GameError> {
        if self.transaction_manager.active_transactions.remove(transaction_id).is_some() {
            self.execute_sql("COMMIT")?;
            
            self.statistics.transactions_committed += 1;
            debug!("提交事务: {}", transaction_id);
            Ok(())
        } else {
            Err(GameError::Database(format!("事务不存在: {}", transaction_id)))
        }
    }
    
    // 回滚事务
    pub fn rollback_transaction(&mut self, transaction_id: &str) -> Result<(), GameError> {
        if self.transaction_manager.active_transactions.remove(transaction_id).is_some() {
            self.execute_sql("ROLLBACK")?;
            
            self.statistics.transactions_rolled_back += 1;
            debug!("回滚事务: {}", transaction_id);
            Ok(())
        } else {
            Err(GameError::Database(format!("事务不存在: {}", transaction_id)))
        }
    }
    
    // 创建查询构建器
    pub fn query_builder(&self) -> QueryBuilder {
        QueryBuilder::new()
    }
    
    // 执行查询构建器
    pub fn execute_query(&mut self, builder: &QueryBuilder) -> Result<QueryResult, GameError> {
        let (query, params) = builder.build()?;
        self.execute(&query, &params)
    }
    
    // 获取统计信息
    pub fn get_statistics(&self) -> &DatabaseStatistics {
        &self.statistics
    }
    
    // 优化数据库
    pub fn optimize(&mut self) -> Result<(), GameError> {
        debug!("开始数据库优化");
        
        // 分析表
        self.execute_sql("ANALYZE")?;
        
        // 如果启用了自动清理，执行VACUUM
        if self.config.auto_vacuum {
            self.execute_sql("VACUUM")?;
        }
        
        // 重建索引
        self.execute_sql("REINDEX")?;
        
        info!("数据库优化完成");
        Ok(())
    }
    
    // 备份数据库
    pub fn backup(&self, backup_path: &str) -> Result<(), GameError> {
        match &self.connection {
            DatabaseConnection::SQLite(conn) => {
                let dest_conn = rusqlite::Connection::open(backup_path)?;
                let backup = rusqlite::backup::Backup::new(conn, &dest_conn)
                    .map_err(|e| GameError::Database(format!("创建备份失败: {}", e)))?;
                
                backup.run_to_completion(5, std::time::Duration::from_millis(250), None)
                    .map_err(|e| GameError::Database(format!("备份执行失败: {}", e)))?;
                
                info!("数据库备份完成: {}", backup_path);
                Ok(())
            }
        }
    }
    
    // 私有方法
    fn create_tables(&mut self) -> Result<(), GameError> {
        let tables = vec![
            ("pokemon_data", "CREATE TABLE IF NOT EXISTS pokemon_data (
                id TEXT PRIMARY KEY,
                data TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )"),
            ("move_data", "CREATE TABLE IF NOT EXISTS move_data (
                id TEXT PRIMARY KEY,
                data TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )"),
            ("item_data", "CREATE TABLE IF NOT EXISTS item_data (
                id TEXT PRIMARY KEY,
                data TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )"),
            ("map_data", "CREATE TABLE IF NOT EXISTS map_data (
                id TEXT PRIMARY KEY,
                data TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )"),
            ("npc_data", "CREATE TABLE IF NOT EXISTS npc_data (
                id TEXT PRIMARY KEY,
                data TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )"),
            ("quest_data", "CREATE TABLE IF NOT EXISTS quest_data (
                id TEXT PRIMARY KEY,
                data TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )"),
        ];
        
        for (table_name, sql) in tables {
            self.execute_sql(sql)?;
            debug!("创建表: {}", table_name);
        }
        
        // 创建索引
        self.create_indexes()?;
        
        Ok(())
    }
    
    fn create_indexes(&mut self) -> Result<(), GameError> {
        let indexes = vec![
            "CREATE INDEX IF NOT EXISTS idx_pokemon_created_at ON pokemon_data(created_at)",
            "CREATE INDEX IF NOT EXISTS idx_pokemon_updated_at ON pokemon_data(updated_at)",
            "CREATE INDEX IF NOT EXISTS idx_move_created_at ON move_data(created_at)",
            "CREATE INDEX IF NOT EXISTS idx_item_created_at ON item_data(created_at)",
            "CREATE INDEX IF NOT EXISTS idx_map_created_at ON map_data(created_at)",
            "CREATE INDEX IF NOT EXISTS idx_npc_created_at ON npc_data(created_at)",
            "CREATE INDEX IF NOT EXISTS idx_quest_created_at ON quest_data(created_at)",
        ];
        
        for sql in indexes {
            self.execute_sql(sql)?;
        }
        
        debug!("索引创建完成");
        Ok(())
    }
    
    fn get_table_name(&self, data_type: DataType) -> String {
        match data_type {
            DataType::Pokemon => "pokemon_data".to_string(),
            DataType::Moves => "move_data".to_string(),
            DataType::Items => "item_data".to_string(),
            DataType::Maps => "map_data".to_string(),
            DataType::NPCs => "npc_data".to_string(),
            DataType::Quests => "quest_data".to_string(),
            _ => format!("{:?}_data", data_type).to_lowercase(),
        }
    }
    
    fn execute_sql(&mut self, sql: &str) -> Result<QueryResult, GameError> {
        let start_time = std::time::Instant::now();
        
        match &mut self.connection {
            DatabaseConnection::SQLite(conn) => {
                let rows_affected = conn.execute(sql, rusqlite::params![])
                    .map_err(|e| GameError::Database(format!("SQL执行失败: {}", e)))?;
                
                let execution_time = start_time.elapsed();
                
                // 更新统计
                self.statistics.queries_executed += 1;
                self.statistics.total_query_time += execution_time;
                self.statistics.average_query_time = 
                    self.statistics.total_query_time / self.statistics.queries_executed as u32;
                
                Ok(QueryResult {
                    data: Vec::new(),
                    rows_affected,
                    execution_time,
                    cached: false,
                })
            }
        }
    }
    
    fn execute(&mut self, query: &str, params: &[QueryValue]) -> Result<QueryResult, GameError> {
        let start_time = std::time::Instant::now();
        
        match &mut self.connection {
            DatabaseConnection::SQLite(conn) => {
                let sqlite_params: Vec<&dyn rusqlite::ToSql> = params.iter()
                    .map(|p| p as &dyn rusqlite::ToSql)
                    .collect();
                
                let rows_affected = conn.execute(query, sqlite_params.as_slice())
                    .map_err(|e| GameError::Database(format!("查询执行失败: {}", e)))?;
                
                let execution_time = start_time.elapsed();
                
                // 更新统计
                self.statistics.queries_executed += 1;
                self.statistics.total_query_time += execution_time;
                
                Ok(QueryResult {
                    data: Vec::new(),
                    rows_affected,
                    execution_time,
                    cached: false,
                })
            }
        }
    }
    
    fn query_single(&mut self, query: &str, params: &[QueryValue]) -> Result<Vec<u8>, GameError> {
        match &mut self.connection {
            DatabaseConnection::SQLite(conn) => {
                let mut stmt = conn.prepare(query)
                    .map_err(|e| GameError::Database(format!("准备查询失败: {}", e)))?;
                
                let sqlite_params: Vec<&dyn rusqlite::ToSql> = params.iter()
                    .map(|p| p as &dyn rusqlite::ToSql)
                    .collect();
                
                let result: String = stmt.query_row(sqlite_params.as_slice(), |row| {
                    Ok(row.get::<_, String>(0)?)
                }).map_err(|e| GameError::Database(format!("查询失败: {}", e)))?;
                
                base64::decode(&result)
                    .map_err(|e| GameError::Database(format!("Base64解码失败: {}", e)))
            }
        }
    }
    
    fn query_multiple(&mut self, query: &str, params: &[QueryValue]) -> Result<Vec<Vec<QueryValue>>, GameError> {
        match &mut self.connection {
            DatabaseConnection::SQLite(conn) => {
                let mut stmt = conn.prepare(query)
                    .map_err(|e| GameError::Database(format!("准备查询失败: {}", e)))?;
                
                let sqlite_params: Vec<&dyn rusqlite::ToSql> = params.iter()
                    .map(|p| p as &dyn rusqlite::ToSql)
                    .collect();
                
                let rows = stmt.query_map(sqlite_params.as_slice(), |row| {
                    let mut result_row = Vec::new();
                    let column_count = row.as_ref().column_count();
                    for i in 0..column_count {
                        let value: rusqlite::types::Value = row.get(i)?;
                        let query_value = self.convert_sqlite_value(value);
                        result_row.push(query_value);
                    }
                    Ok(result_row)
                }).map_err(|e| GameError::Database(format!("查询失败: {}", e)))?;
                
                let mut results = Vec::new();
                for row in rows {
                    results.push(row.map_err(|e| GameError::Database(format!("行处理失败: {}", e)))?);
                }
                
                Ok(results)
            }
        }
    }
    
    fn convert_sqlite_value(&self, value: rusqlite::types::Value) -> QueryValue {
        match value {
            rusqlite::types::Value::Null => QueryValue::Null,
            rusqlite::types::Value::Integer(i) => QueryValue::Integer(i),
            rusqlite::types::Value::Real(f) => QueryValue::Float(f),
            rusqlite::types::Value::Text(s) => QueryValue::String(s),
            rusqlite::types::Value::Blob(b) => QueryValue::String(base64::encode(&b)),
        }
    }
}

impl TransactionManager {
    pub fn new() -> Self {
        Self {
            active_transactions: HashMap::new(),
            transaction_counter: 0,
        }
    }
    
    pub fn begin_transaction(&mut self, isolation_level: IsolationLevel) -> String {
        self.transaction_counter += 1;
        let transaction_id = format!("tx_{}", self.transaction_counter);
        
        let transaction = Transaction {
            id: transaction_id.clone(),
            started_at: std::time::Instant::now(),
            isolation_level,
            read_only: false,
            statements: Vec::new(),
        };
        
        self.active_transactions.insert(transaction_id.clone(), transaction);
        transaction_id
    }
}

impl QueryBuilder {
    pub fn new() -> Self {
        Self {
            query_type: QueryType::Select,
            table: String::new(),
            columns: Vec::new(),
            conditions: Vec::new(),
            joins: Vec::new(),
            order_by: Vec::new(),
            group_by: Vec::new(),
            having: Vec::new(),
            limit: None,
            offset: None,
        }
    }
    
    pub fn select(mut self, columns: &[&str]) -> Self {
        self.query_type = QueryType::Select;
        self.columns = columns.iter().map(|s| s.to_string()).collect();
        self
    }
    
    pub fn from(mut self, table: &str) -> Self {
        self.table = table.to_string();
        self
    }
    
    pub fn where_clause(mut self, column: &str, op: ComparisonOperator, value: QueryValue) -> Self {
        self.conditions.push(Condition {
            column: column.to_string(),
            operator: op,
            value,
            logical_op: None,
        });
        self
    }
    
    pub fn and(mut self, column: &str, op: ComparisonOperator, value: QueryValue) -> Self {
        self.conditions.push(Condition {
            column: column.to_string(),
            operator: op,
            value,
            logical_op: Some(LogicalOperator::And),
        });
        self
    }
    
    pub fn order_by(mut self, column: &str, direction: SortDirection) -> Self {
        self.order_by.push(OrderBy {
            column: column.to_string(),
            direction,
        });
        self
    }
    
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }
    
    pub fn build(&self) -> Result<(String, Vec<QueryValue>), GameError> {
        let mut query = String::new();
        let mut params = Vec::new();
        
        match self.query_type {
            QueryType::Select => {
                query.push_str("SELECT ");
                if self.columns.is_empty() {
                    query.push('*');
                } else {
                    query.push_str(&self.columns.join(", "));
                }
                query.push_str(&format!(" FROM {}", self.table));
            },
            _ => return Err(GameError::Database("暂不支持的查询类型".to_string())),
        }
        
        // WHERE子句
        if !self.conditions.is_empty() {
            query.push_str(" WHERE ");
            for (i, condition) in self.conditions.iter().enumerate() {
                if i > 0 {
                    if let Some(logical_op) = condition.logical_op {
                        match logical_op {
                            LogicalOperator::And => query.push_str(" AND "),
                            LogicalOperator::Or => query.push_str(" OR "),
                        }
                    }
                }
                
                query.push_str(&format!("{} {} ?", condition.column, self.operator_to_sql(condition.operator)));
                params.push(condition.value.clone());
            }
        }
        
        // ORDER BY子句
        if !self.order_by.is_empty() {
            query.push_str(" ORDER BY ");
            let order_clauses: Vec<String> = self.order_by.iter()
                .map(|o| format!("{} {}", o.column, if o.direction == SortDirection::Ascending { "ASC" } else { "DESC" }))
                .collect();
            query.push_str(&order_clauses.join(", "));
        }
        
        // LIMIT子句
        if let Some(limit) = self.limit {
            query.push_str(&format!(" LIMIT {}", limit));
        }
        
        Ok((query, params))
    }
    
    fn operator_to_sql(&self, op: ComparisonOperator) -> &'static str {
        match op {
            ComparisonOperator::Equal => "=",
            ComparisonOperator::NotEqual => "!=",
            ComparisonOperator::Greater => ">",
            ComparisonOperator::GreaterEqual => ">=",
            ComparisonOperator::Less => "<",
            ComparisonOperator::LessEqual => "<=",
            ComparisonOperator::Like => "LIKE",
            ComparisonOperator::In => "IN",
            ComparisonOperator::NotIn => "NOT IN",
            ComparisonOperator::IsNull => "IS NULL",
            ComparisonOperator::IsNotNull => "IS NOT NULL",
        }
    }
}

impl DatabaseConfig {
    pub fn default(db_path: &str) -> Self {
        Self {
            db_path: db_path.to_string(),
            max_connections: 10,
            connection_timeout: std::time::Duration::from_secs(30),
            query_timeout: std::time::Duration::from_secs(60),
            auto_vacuum: true,
            cache_size: 2000,
            journal_mode: JournalMode::WAL,
            synchronous: SynchronousMode::Normal,
        }
    }
}

impl rusqlite::ToSql for QueryValue {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        match self {
            QueryValue::String(s) => Ok(rusqlite::types::ToSqlOutput::from(s.as_str())),
            QueryValue::Integer(i) => Ok(rusqlite::types::ToSqlOutput::from(*i)),
            QueryValue::Float(f) => Ok(rusqlite::types::ToSqlOutput::from(*f)),
            QueryValue::Boolean(b) => Ok(rusqlite::types::ToSqlOutput::from(*b)),
            QueryValue::Null => Ok(rusqlite::types::ToSqlOutput::from(rusqlite::types::Null)),
            QueryValue::Array(_) => Err(rusqlite::Error::ToSqlConversionFailure(
                Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "数组类型不支持直接转换"))
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use serde::{Deserialize, Serialize};
    
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct TestData {
        name: String,
        value: i32,
    }
    
    #[test]
    fn test_database_creation() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        
        let db = GameDatabase::new(db_path.to_str().unwrap());
        assert!(db.is_ok());
    }
    
    #[test]
    fn test_save_and_load_data() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let mut db = GameDatabase::new(db_path.to_str().unwrap()).unwrap();
        
        let test_data = TestData {
            name: "Test Pokemon".to_string(),
            value: 42,
        };
        
        // 保存数据
        db.save_data(DataType::Pokemon, "test_id", &test_data).unwrap();
        
        // 加载数据
        let loaded_data: TestData = db.load_data(DataType::Pokemon, "test_id").unwrap();
        assert_eq!(loaded_data, test_data);
    }
    
    #[test]
    fn test_query_builder() {
        let builder = QueryBuilder::new()
            .select(&["id", "name"])
            .from("pokemon_data")
            .where_clause("level", ComparisonOperator::Greater, QueryValue::Integer(10))
            .and("type", ComparisonOperator::Equal, QueryValue::String("Fire".to_string()))
            .order_by("name", SortDirection::Ascending)
            .limit(50);
        
        let (query, params) = builder.build().unwrap();
        assert!(query.contains("SELECT id, name FROM pokemon_data"));
        assert!(query.contains("WHERE level > ?"));
        assert!(query.contains("AND type = ?"));
        assert!(query.contains("ORDER BY name ASC"));
        assert!(query.contains("LIMIT 50"));
        assert_eq!(params.len(), 2);
    }
    
    #[test]
    fn test_transaction_manager() {
        let mut manager = TransactionManager::new();
        
        let tx_id = manager.begin_transaction(IsolationLevel::ReadCommitted);
        assert!(manager.active_transactions.contains_key(&tx_id));
        assert_eq!(manager.active_transactions.len(), 1);
        
        let tx = manager.active_transactions.get(&tx_id).unwrap();
        assert_eq!(tx.isolation_level, IsolationLevel::ReadCommitted);
    }
}