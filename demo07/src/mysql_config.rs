use mysql::*;

/**************************************************************
* Description: 数据库配置
* Author: yuanhao
* Versions: V1
**************************************************************/
pub struct Mysql {
    pool: Pool,
}
/**************************************************************
* Description: 基本方法
* Author: yuanhao
* Versions: V1
**************************************************************/
impl Mysql {
    /**************************************************************
     * Description: 获取连接池配置
     * Author: yuanhao
     * Versions: V1
     **************************************************************/
    pub fn new() -> Self {
        let url = "mysql://root:root@localhost:3306/nacos_config";
        Self {
            pool: Pool::new(url).unwrap(),
        }
    }
    pub fn get_conn(self) -> Result<PooledConn> {
        self.pool.get_conn()
    }
}
