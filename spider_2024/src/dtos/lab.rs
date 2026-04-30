use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize, Debug)]
#[expect(non_snake_case)]
pub struct LabLoginInfoRes {
    /// -1 表示账号或密码错误，1 表示登录成功
    pub RTNCode: i32,
    /// 这个字段有可能是 string（当登录失败时），也有可能是 object（当登录成功时）
    pub Data: Value,
}

#[derive(Deserialize, Debug)]
#[expect(non_snake_case)]
pub struct LabArrangeItem {
    /// 座位号
    pub SeatNo: String,
    /// 实验名称
    pub LabName: String,
    /// 课程名称
    pub CourseName: String,
    /// 上课老师名称
    pub UserName: String,
    /// 上课周次
    pub Weeks: String,
    /// 上课星期几
    pub WeekName: String,
    /// 上课日期，格式如“2025/9/27 0:00:00”目前来看就前面的日期部分正确
    pub ClassDate: String,
    /// 上课开始时间
    pub StartTime: String,
    /// 上课地点
    pub ClassRoom: String,
    /// 联系电话
    pub MobileNum: String,
    /// 联系邮箱
    pub Email: String,
}

#[derive(Deserialize, Debug)]
pub struct LabSemInfoRes {
    pub id: String,
    pub text: String,
}

#[derive(Deserialize, Debug)]
#[expect(non_snake_case)]
pub struct LabScoreItem {
    /// 出勤情况
    pub AttendanceName: String,
    /// 实验名称
    pub LabName: String,
    /// 实验成绩，没有成绩的话是空字符串
    pub LabScore: String,
    /// 实验id
    pub LabID: String,
    /// 上课地点，这个字段只是用来判断是否为虚拟实验的
    pub ClassRoom: String,
}

#[derive(Deserialize, Debug)]
#[expect(non_snake_case)]
pub struct LabScoreDetailItem {
    /// 对应的成绩结构id
    pub LabScoreStructureID: i32,
    /// 对应的实验id
    pub LabID: i32,
    /// 分数
    pub LabStructureScore: Option<f64>,
}

#[derive(Deserialize, Debug)]
#[expect(non_snake_case)]
pub struct LabScoreStructureItem {
    /// 成绩结构id
    pub LabScoreStructureID: i32,
    /// 成绩结构名称
    pub LabScoreStructureName: String,
}

#[derive(Deserialize, Debug)]
#[expect(non_snake_case)]
pub struct LabCourseItem {
    /// 课程名称
    pub CourseName: String,
    /// 课程成绩，没有成绩的话是空字符串
    pub CourseFinalScore: String,
    /// 课程id
    pub CourseID: String,
}

#[derive(Deserialize, Debug)]
#[expect(non_snake_case)]
pub struct VirtualLabGradeItem {
    /// 实验名称
    pub LabName: String,
    /// 实验成绩，没有成绩的话是空字符串
    pub LabScore: String,
}
