use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub struct GradeReq {
    pub stu_id: String,
    pub xn: u16,
    pub xq: u8,
}

/// 教务 `考试成绩 > 课程成绩` 返回数据单项
#[derive(Deserialize, Debug)]
pub struct GradeInfoRes {
    // 未知字段
    // pub cj0708id: String,
    // 学年学期信息（暂时不用）
    // pub xnxqid: String,
    /// 课程代码
    pub kch: String,
    /// 课程名称
    pub kc_mc: String,
    // 开课学院（暂时不用）
    // pub ksdw: String,
    //  似乎和 xnxqid 重复
    // pub xqmc: String,
    /// 学分
    pub xf: f32,
    // 总学时（暂时不用）
    // pub zxs: u32,
    // 考试方式（暂时不用）
    // pub ksfs: String,
    /// 课程属性（必修/选修等）
    pub kcsx: Option<String>,
    // 似乎又和 xnxqid 重复
    // pub xqstr: String,
    /// 总成绩
    pub zcj: u8,
    // 总成绩字符串形式（暂时不用）
    // pub zcjstr: String,
    // 未知字段
    // pub kz: u8,
    ///  课程性质（通识必修/专业核心等）
    pub kcxzmc: String,
    // 未知字段
    // pub xs0101id: String,
    /// 用于课程成绩详情查询，部分成绩没有该字段
    pub jx0404id: Option<String>,
    /// 绩点
    pub jd: f32,
    // 考试性质（暂时不用）
    // pub ksxz: String,
    /// 主修还是辅修
    pub falb: String,
    /// 成绩标识（缓考/重修等，注意这个标识是挂在为 0 分的那个成绩 item 上）
    pub cjbs: Option<String>,
}

#[derive(Deserialize, Debug, Serialize)]
#[expect(dead_code)]
struct GradeRank {
    /// 算术平均成绩排名
    pub arithmetic_rank: String,
    /// 算术平均成绩
    pub arithmetic_score: String,
    /// 加权平均成绩排名
    pub weighted_rank: String,
    /// 加权平均成绩
    pub weighted_score: String,
    /// GPA排名
    pub gpa_rank: String,
    /// GPA
    pub gpa: String,
}

/// 教务 `教学运行 > 我的课表 > 有课表课程` 返回数据单项
/// 还有其他一些具体学时信息的字段，懒得搞了
#[derive(Deserialize, Debug)]
pub struct CourseInfoRes {
    /// 课程代码
    pub kch: String,
    /// 课程名称
    pub kc_mc: String,
    /// 教师名称
    pub jg0101mc: String,
    /// 教师工号（暂时不用）
    pub jsgh: String,
    pub kt_mc: String, // 上课班级
    /// 课堂容量（暂时不用）
    pub pkrs: u16,
    /// 上课人数
    pub xkrs: u16,
    /// 课程性质（通识必修/专业核心等）
    pub kcxz: String,
    /// 课程类别（必修/选修等）
    pub kclb: String,
    /// 通知单编号（暂时不用）
    pub jx0404id: String,
    /// 分组名称，这里当作课程的备注信息
    pub fzmc: Option<String>,
    /// 上课时间
    pub sktime: String,
    /// 上课地点
    pub skddmc: String,
    /// 上课校区
    pub skxqmc: String,
    /// 开课院系（暂时不用）
    pub kkyx: String,
    /// 周学时（暂时不用）
    pub zhouxs: String,
    /// 学分
    pub xf: f32,
    /// 总学时（暂时不用）
    pub zxs: u16,
    /// 考核方式（暂时不用）
    pub khfs: String,
}

/// 教务 `教学运行 > 我的课表 > 无课表课程` 返回数据单项
#[derive(Deserialize, Debug)]
pub struct ExtraCourseInfoRes {
    /// 课程代码
    pub kch: String,
    /// 课程名称
    pub kc_mc: String,
    /// 教师名称
    pub jg0101mc: String,
    /// 分组名称
    pub fzmc: Option<String>,
    /// 课程性质（通识必修/专业核心等）
    pub kcxz: String,
    /// 上课班级
    pub kt_mc: String,
    /// 上课人数
    pub xkrs: u16,
    /// 上课校区
    pub skxqmc: String,
    /// 学分
    pub xf: f32,
}

#[derive(Deserialize, Debug)]
pub struct EmptyRoomReq {
    pub stu_id: String, // 旧爬虫不需要学号信息，但是新爬虫决定让各自账号去各自请求空教室信息
    pub build_id: String,
    pub day: u8,
    /// 节次信息
    pub jc: String,
    pub week: String,
    pub xn: u16,
    pub xq: u8,
}

// 注释见前端
#[derive(Debug, Deserialize)]
pub struct HdjwGradeRankReq {
    pub stu_id: String,
    pub year: Option<u16>,
    pub term: Option<u8>,
    pub course: u8,
    pub rank: u8,
}

#[derive(Serialize, Debug, Deserialize)]
pub struct RankRes {
    pub score: String,
    pub rank: String,
}

/// 考试安排单项
/// 带 Option 的字段应该是类似于体育理论这样考试安排信息很不全的课程
#[derive(Deserialize, Debug)]
pub struct ExamArrangeItemRes {
    /// 课程代码
    pub kch: String,
    /// 课程名称
    pub kskcmc: String,
    /// 考试校区
    pub ksxq: Option<String>,
    /// 考试的教室
    pub js_mc: Option<String>,
    /// 考试时间（已经是一个时间区间了）
    pub kssj: Option<String>,
    /// 座位号
    pub zwh: Option<String>,
}
