use anyhow::anyhow;
use chrono::Datelike;
use serde_json::Value;

use crate::{
    dtos::hdjw::{
        CourseInfoRes, EmptyRoomReq, ExamArrangeItemRes,
        ExtraCourseInfoRes, GradeInfoRes, GradeReq, HdjwGradeRankReq,
        RankRes,
    },
    spiders::{self},
};

pub async fn get_grade_handler(
    req: GradeReq,
) -> Result<Vec<GradeInfoRes>, crate::Error> {
    let mut raw_res =
        spiders::hdjw::get_grade(&req.stu_id, req.xn, req.xq).await?;

    // 取 data 字段返回
    let res: Vec<GradeInfoRes> =
        serde_json::from_value(raw_res["data"].take())
            .map_err(|e| anyhow!("成绩数据解析错误 {}", e))?;

    Ok(res)
}

pub async fn get_empty_classroom_handler(
    req: EmptyRoomReq,
) -> Result<Value, crate::Error> {
    let res = spiders::hdjw::get_empty_classroom(
        &req.stu_id,
        req.xn,
        req.xq,
        &req.week,
        req.day,
        &req.jc,
        &req.build_id,
    )
    .await?;
    Ok(res)
}

pub async fn get_class_table_handler(
    req: GradeReq,
) -> Result<Vec<CourseInfoRes>, crate::Error> {
    // 如果学号第一个是S或者B，就是属于研究生系统
    // TODO: 研究生系统
    if req.stu_id.starts_with('S') || req.stu_id.starts_with('B') {
        let mut raw_res = spiders::graduate::get_class_table(
            &req.stu_id,
            req.xn,
            req.xq,
        )
        .await?;

        // 取 data 字段返回
        let res: Vec<CourseInfoRes> =
            serde_json::from_value(raw_res["data"].take())
                .map_err(|e| anyhow!("获取课表数据失败 {}", e))?;

        return Ok(res);
    }

    let mut raw_res =
        spiders::hdjw::get_class_table(&req.stu_id, req.xn, req.xq)
            .await?;

    match raw_res.get("count").and_then(|c| c.as_u64()) {
        None => Err(anyhow!("获取课表数据失败").into()),
        Some(0) => Ok(vec![]), // 有可能 count 是 0 但是不带 data 字段
        Some(_) => {
            // 取 data 字段返回
            let res: Vec<CourseInfoRes> =
                serde_json::from_value(raw_res["data"].take())
                    .map_err(|e| anyhow!("获取课表数据失败 {}", e))?;

            Ok(res)
        }
    }
}

pub async fn get_exam_schedule_handler(
    req: GradeReq,
) -> Result<Vec<ExamArrangeItemRes>, crate::Error> {
    let mut raw_res =
        spiders::hdjw::get_exam_schedule(&req.stu_id, req.xn, req.xq)
            .await?;

    let res: Vec<ExamArrangeItemRes> =
        serde_json::from_value(raw_res["data"].take())
            .map_err(|e| anyhow!("获取考试安排失败 {}", e))?;
    Ok(res)
}

pub async fn get_rank_from_hdjw_handler(
    req: HdjwGradeRankReq,
) -> Result<RankRes, crate::Error> {
    let enter_year = req.stu_id[0..4]
        .parse::<u16>()
        .map_err(|_| anyhow!("暂时仅支持本科生"))?;
    let mut selection = Vec::new();
    if let Some(year) = req.year {
        if let Some(term) = req.term {
            selection.push(format!("{}-{}-{}", year, year + 1, term));
        } else {
            selection.push(format!("{}-{}-1", year, year + 1));
            selection.push(format!("{}-{}-2", year, year + 1));
            selection.push(format!("{}-{}-3", year, year + 1));
        }
    } else {
        // 从入学年份查到当前年份，多查了没关系
        let current_year = chrono::Local::now().year() as u16;
        for i in enter_year..=current_year {
            selection.push(format!("{}-{}-1", i, i + 1));
            selection.push(format!("{}-{}-2", i, i + 1));
            selection.push(format!("{}-{}-3", i, i + 1));
        }
    }
    let range = match req.course {
        1 => {
            "01,02,03,04,05,06,07,08,09,10,11,12,13,14,15,16,17,18,88"
        }
        2 => "01,02,03,04,08,10,11,12,16",
        3 => {
            if enter_year >= 2024 {
                // 2024 级开始实行不同的核心课方案
                "03,16"
            } else {
                "08,12,16"
            }
        }
        _ => return Err(anyhow!("course参数错误").into()),
    };
    let rank = match req.rank {
        1 => 4,
        2 => 2,
        3 => 3,
        _ => return Err(anyhow!("rank参数错误").into()),
    };
    let res = spiders::hdjw::get_grade_rank_common(
        &req.stu_id,
        &selection,
        range.to_string(),
        rank,
    )
    .await?;
    Ok(RankRes {
        score: res.1,
        rank: res.0,
    })
}

pub async fn get_grade_from_ca_handler(
    stu_id: &str,
) -> Result<String, crate::Error> {
    let res = spiders::hdjw::get_grade_from_ca(stu_id).await?;
    Ok(res)
}

pub async fn get_grade_detail_handler(
    stu_id: &str,
    jx0404id: &str,
) -> Result<String, crate::Error> {
    let res =
        spiders::hdjw::get_grade_detail(stu_id, jx0404id).await?;
    Ok(res)
}

pub async fn get_class_table_extra_handler(
    req: GradeReq,
) -> Result<Vec<ExtraCourseInfoRes>, crate::Error> {
    let mut raw_res = spiders::hdjw::get_class_table_extra(
        &req.stu_id,
        req.xn,
        req.xq,
    )
    .await?;

    match raw_res.get("count").and_then(|c| c.as_u64()) {
        None => Err(anyhow!("获取无课表课程失败").into()),
        Some(0) => Ok(vec![]), // 有可能 count 是 0 但是不带 data 字段
        Some(_) => {
            // 取 data 字段返回
            let res: Vec<ExtraCourseInfoRes> =
                serde_json::from_value(raw_res["data"].take())
                    .map_err(|e| {
                        anyhow!("获取无课表课程失败 {}", e)
                    })?;

            Ok(res)
        }
    }
}
