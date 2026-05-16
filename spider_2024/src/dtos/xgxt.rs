use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct PersonInfo {
    // 姓名
    pub name: String,
    // 性别
    pub gender: String,
    // 政治面貌
    pub politic: String,
    // 民族
    pub race: String,
    // 籍贯
    pub hometown: String,
    // 培养层次，本科/研究生/博士生
    pub level: String,
    // 学院
    pub academy: String,
    // 专业
    pub major: String,
    // 班级
    pub class: String,
    // 寝室楼
    pub dormitory: String,
    // 房间号
    pub room: String,
    // 手机号
    pub phone: String,
    // 微信号
    pub wechat: String,
    // qq号
    pub qq: String,
    // 电子邮箱
    pub email: String,
    // 年级（入学年份应该与年级相等）
    pub enter_year: u16,
    // 学制
    pub xz: u8,
    // 学号
    pub stu_id: String,
}
