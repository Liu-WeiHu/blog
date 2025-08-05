use serde::{Deserialize, Serialize};
use sqlx::{
    postgres::{PgArgumentBuffer, PgHasArrayType, PgTypeInfo, PgValueRef},
    prelude::FromRow,
};

// 定义 UserExt 结构体，用于与 user_ext 表交互
#[derive(Debug, Serialize, Deserialize, Default, Clone, FromRow)]
pub struct UserExt {
    pub id: i32,
    pub user_id: i32,
    pub age: Option<i32>,
    pub gender: Option<Gender>,
    pub education: Option<Education>,
    pub hometown: Option<String>,
    pub address: Option<String>,
}

// 定义 Gender 枚举
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum Gender {
    Male,
    Female,
    Other,
}

impl sqlx::Type<sqlx::Postgres> for Gender {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("gender_type")
    }
}

impl<'r> sqlx::Decode<'r, sqlx::Postgres> for Gender {
    fn decode(value: PgValueRef<'r>) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let s = <&str as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        match s {
            "male" => Ok(Gender::Male),
            "female" => Ok(Gender::Female),
            "other" => Ok(Gender::Other),
            _ => Err(s.into()),
        }
    }
}

impl<'q> sqlx::Encode<'q, sqlx::Postgres> for Gender {
    fn encode_by_ref(
        &self,
        buf: &mut PgArgumentBuffer,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        let s = match self {
            Gender::Male => "male",
            Gender::Female => "female",
            Gender::Other => "other",
        };
        <&str as sqlx::Encode<sqlx::Postgres>>::encode(s, buf)
    }
}

impl PgHasArrayType for Gender {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("_gender_type")
    }
}

// 定义 Education 枚举
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum Education {
    Primary,
    Secondary,
    Bachelor,
    Master,
    Doctorate,
    Other,
}

impl sqlx::Type<sqlx::Postgres> for Education {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("education_type")
    }
}

impl<'r> sqlx::Decode<'r, sqlx::Postgres> for Education {
    fn decode(
        value: sqlx::postgres::PgValueRef<'r>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let s = <&str as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        match s {
            "primary" => Ok(Education::Primary),
            "secondary" => Ok(Education::Secondary),
            "bachelor" => Ok(Education::Bachelor),
            "master" => Ok(Education::Master),
            "doctorate" => Ok(Education::Doctorate),
            "other" => Ok(Education::Other),
            _ => Err(s.into()),
        }
    }
}

impl<'q> sqlx::Encode<'q, sqlx::Postgres> for Education {
    fn encode_by_ref(
        &self,
        buf: &mut sqlx::postgres::PgArgumentBuffer,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        let s = match self {
            Education::Primary => "primary",
            Education::Secondary => "secondary",
            Education::Bachelor => "bachelor",
            Education::Master => "master",
            Education::Doctorate => "doctorate",
            Education::Other => "other",
        };
        <&str as sqlx::Encode<sqlx::Postgres>>::encode(s, buf)
    }
}

impl PgHasArrayType for Education {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("_education_type")
    }
}
