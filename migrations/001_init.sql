-- HnuDoc 数据库结构初始化
-- 字符集采用 utf8mb4 以支持中文/emoji
-- 时间字段统一使用 DATETIME（存 UTC+8 本地时间，与参考后端一致）

CREATE DATABASE IF NOT EXISTS `hnudoc`
    DEFAULT CHARACTER SET utf8mb4
    DEFAULT COLLATE utf8mb4_unicode_ci;

USE `hnudoc`;

-- ============================================================
-- 用户表
-- ============================================================
CREATE TABLE IF NOT EXISTS `users` (
    `stu_id`      VARCHAR(32)  NOT NULL COMMENT '学号（已大写、去空白）',
    `name`        VARCHAR(64)  NOT NULL DEFAULT '' COMMENT '姓名',
    -- 加密后的密码（AES-256-CBC，与参考后端 utils::crypto 一致）
    `password`    TEXT         NOT NULL COMMENT '加密后的个人门户密码',
    -- 权限以 JSON 数组形式存储，例如 ["search","download","upload","review"]
    `permissions` JSON         NOT NULL,
    `created_at`  DATETIME     NOT NULL,
    `updated_at`  DATETIME     NOT NULL,
    PRIMARY KEY (`stu_id`)
) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_unicode_ci
    COMMENT '试卷库用户';

-- ============================================================
-- 试卷（Document）
-- ============================================================
CREATE TABLE IF NOT EXISTS `documents` (
    `id`         INT UNSIGNED NOT NULL AUTO_INCREMENT,
    `name`       VARCHAR(255) NOT NULL COMMENT '资料名称（一般是课程名）',
    `typ`        VARCHAR(16)  NOT NULL COMMENT 'final / mid / other',
    -- date 字段允许 NULL 表示未知年份
    -- date_typ: year / semester / grade
    `date_typ`   VARCHAR(16)  NULL,
    `date_year`  INT          NULL,
    `answer`     TINYINT(1)   NOT NULL DEFAULT 0 COMMENT '是否含答案',
    `page`       INT UNSIGNED NOT NULL DEFAULT 0 COMMENT '页数',
    `tags`       JSON         NOT NULL,
    `comment`    TEXT         NULL,
    `md5`        CHAR(32)     NOT NULL COMMENT '文件 md5',
    `categories` JSON         NOT NULL COMMENT '分类，如 ["A1","A2"]',
    `file_path`  VARCHAR(512) NOT NULL COMMENT '本地文件相对路径',
    `created_at` DATETIME     NOT NULL,
    PRIMARY KEY (`id`),
    UNIQUE KEY `uk_md5` (`md5`),
    KEY `idx_typ` (`typ`),
    KEY `idx_name` (`name`)
) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_unicode_ci
    COMMENT '正式收录的试卷';

-- ============================================================
-- 试卷集
-- ============================================================
CREATE TABLE IF NOT EXISTS `collections` (
    `id`          INT UNSIGNED NOT NULL AUTO_INCREMENT,
    `name`        VARCHAR(255) NOT NULL,
    `description` TEXT         NOT NULL,
    `created_at`  DATETIME     NOT NULL,
    PRIMARY KEY (`id`)
) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_unicode_ci
    COMMENT '试卷集（专题）';

CREATE TABLE IF NOT EXISTS `collection_items` (
    `collection_id` INT UNSIGNED NOT NULL,
    `document_id`   INT UNSIGNED NOT NULL,
    `sort_order`    INT          NOT NULL DEFAULT 0,
    PRIMARY KEY (`collection_id`, `document_id`),
    KEY `idx_doc` (`document_id`),
    CONSTRAINT `fk_ci_col`
        FOREIGN KEY (`collection_id`) REFERENCES `collections` (`id`)
        ON DELETE CASCADE,
    CONSTRAINT `fk_ci_doc`
        FOREIGN KEY (`document_id`) REFERENCES `documents` (`id`)
        ON DELETE CASCADE
) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_unicode_ci
    COMMENT '试卷集与试卷的关联';

-- ============================================================
-- 待审核试卷
-- ============================================================
CREATE TABLE IF NOT EXISTS `pending_documents` (
    `id`          INT UNSIGNED NOT NULL AUTO_INCREMENT,
    `name`        VARCHAR(255) NOT NULL,
    `typ`         VARCHAR(16)  NOT NULL,
    `date_typ`    VARCHAR(16)  NULL,
    `date_year`   INT          NULL,
    `answer`      TINYINT(1)   NOT NULL DEFAULT 0,
    `page`        INT UNSIGNED NOT NULL DEFAULT 0,
    `tags`        JSON         NOT NULL,
    -- 上传时填写的备注
    `comment`     TEXT         NULL,
    `md5`         CHAR(32)     NOT NULL,
    `categories`  JSON         NOT NULL,
    `file_path`   VARCHAR(512) NOT NULL,
    -- 审核状态
    `status`      VARCHAR(16)  NOT NULL DEFAULT 'pending'
        COMMENT 'pending / accepted / rejected',
    `stu_id`      VARCHAR(32)  NOT NULL COMMENT '提交者学号',
    -- 审核意见（拒绝时填写）
    `audit_comment` TEXT       NULL,
    `target`      INT UNSIGNED NULL COMMENT '通过后对应的 documents.id',
    `create_time` DATETIME     NOT NULL,
    `update_time` DATETIME     NOT NULL,
    PRIMARY KEY (`id`),
    KEY `idx_status` (`status`),
    KEY `idx_stu` (`stu_id`),
    KEY `idx_md5` (`md5`),
    CONSTRAINT `fk_pending_target`
        FOREIGN KEY (`target`) REFERENCES `documents` (`id`)
        ON DELETE SET NULL
) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_unicode_ci
    COMMENT '待审核 / 已审核试卷';
