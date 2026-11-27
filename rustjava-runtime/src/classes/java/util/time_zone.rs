use alloc::vec;

use jvm::{Array, ClassInstanceRef, Jvm, Result, runtime::JavaLangString};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};

use crate::{
    RuntimeClassProto, RuntimeContext,
    classes::java::lang::{Object, String},
};

const HOUR: i32 = 3_600_000;

/// The zones `getTimeZone` knows by ID, with their offsets from GMT in milliseconds. `JST` is Java's
/// three-letter ID for nine hours east; `KST` is the ID runtimes built for Korea give the same offset.
const ZONES: [(&str, i32); 4] = [("GMT", 0), ("UTC", 0), ("JST", 9 * HOUR), ("KST", 9 * HOUR)];

// abstract class java.util.TimeZone
pub struct TimeZone;

impl TimeZone {
    pub fn as_proto() -> RuntimeClassProto {
        RuntimeClassProto {
            name: "java/util/TimeZone",
            parent_class: Some("java/lang/Object"),
            interfaces: vec!["java/io/Serializable", "java/lang/Cloneable"],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::PROTECTED),
                JavaMethodProto::new(
                    "getTimeZone",
                    "(Ljava/lang/String;)Ljava/util/TimeZone;",
                    Self::get_time_zone,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "getDefault",
                    "()Ljava/util/TimeZone;",
                    Self::get_default,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC | MethodAccessFlags::SYNCHRONIZED,
                ),
                JavaMethodProto::new(
                    "setDefault",
                    "(Ljava/util/TimeZone;)V",
                    Self::set_default,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC | MethodAccessFlags::SYNCHRONIZED,
                ),
                JavaMethodProto::new(
                    "getAvailableIDs",
                    "()[Ljava/lang/String;",
                    Self::get_available_ids,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new("getID", "()Ljava/lang/String;", Self::get_id, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setID", "(Ljava/lang/String;)V", Self::set_id, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("clone", "()Ljava/lang/Object;", Self::clone, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new_abstract("getOffset", "(IIIIII)I", MethodAccessFlags::PUBLIC | MethodAccessFlags::ABSTRACT),
                JavaMethodProto::new_abstract("getRawOffset", "()I", MethodAccessFlags::PUBLIC | MethodAccessFlags::ABSTRACT),
                JavaMethodProto::new_abstract("setRawOffset", "(I)V", MethodAccessFlags::PUBLIC | MethodAccessFlags::ABSTRACT),
                JavaMethodProto::new_abstract("useDaylightTime", "()Z", MethodAccessFlags::PUBLIC | MethodAccessFlags::ABSTRACT),
                JavaMethodProto::new_abstract(
                    "inDaylightTime",
                    "(Ljava/util/Date;)Z",
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::ABSTRACT,
                ),
            ],
            fields: vec![
                JavaFieldProto::new("ID", "Ljava/lang/String;", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new(
                    "defaultTimeZone",
                    "Ljava/util/TimeZone;",
                    FieldAccessFlags::PRIVATE | FieldAccessFlags::STATIC,
                ),
            ],
            access_flags: ClassAccessFlags::PUBLIC | ClassAccessFlags::ABSTRACT,
        }
    }

    async fn init(jvm: &Jvm, _: &mut RuntimeContext, mut this: ClassInstanceRef<Self>) -> Result<()> {
        tracing::debug!("java.util.TimeZone::<init>({this:?})");

        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;
        let id = JavaLangString::from_rust_string(jvm, "GMT").await?;
        jvm.put_field(&mut this, "java/util/TimeZone", "ID", "Ljava/lang/String;", id).await
    }

    async fn get_time_zone(jvm: &Jvm, _: &mut RuntimeContext, id: ClassInstanceRef<String>) -> Result<ClassInstanceRef<Self>> {
        tracing::debug!("java.util.TimeZone::getTimeZone({id:?})");

        if id.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "ID").await);
        }

        let requested_id = JavaLangString::to_rust_string(jvm, &id).await?;
        let Some((_, raw_offset)) = ZONES.iter().find(|(zone_id, _)| *zone_id == requested_id) else {
            return Self::gmt(jvm).await;
        };
        let result = jvm
            .new_class("java/util/SimpleTimeZone", "(ILjava/lang/String;)V", (*raw_offset, id))
            .await?;
        Ok(result.into())
    }

    async fn gmt(jvm: &Jvm) -> Result<ClassInstanceRef<Self>> {
        let id = JavaLangString::from_rust_string(jvm, "GMT").await?;
        let result = jvm.new_class("java/util/SimpleTimeZone", "(ILjava/lang/String;)V", (0i32, id)).await?;
        Ok(result.into())
    }

    /// The zone the `user.timezone` system property names, GMT when it names none.
    async fn host_zone(jvm: &Jvm) -> Result<ClassInstanceRef<Self>> {
        let key = JavaLangString::from_rust_string(jvm, "user.timezone").await?;
        let id: ClassInstanceRef<String> = jvm
            .invoke_static("java/lang/System", "getProperty", "(Ljava/lang/String;)Ljava/lang/String;", (key,))
            .await?;
        if id.is_null() {
            return Self::gmt(jvm).await;
        }

        jvm.invoke_static("java/util/TimeZone", "getTimeZone", "(Ljava/lang/String;)Ljava/util/TimeZone;", (id,))
            .await
    }

    async fn get_default(jvm: &Jvm, _: &mut RuntimeContext) -> Result<ClassInstanceRef<Self>> {
        tracing::debug!("java.util.TimeZone::getDefault()");

        let mut default: ClassInstanceRef<Self> = jvm
            .get_static_field("java/util/TimeZone", "defaultTimeZone", "Ljava/util/TimeZone;")
            .await?;
        if default.is_null() {
            default = Self::host_zone(jvm).await?;
            jvm.put_static_field("java/util/TimeZone", "defaultTimeZone", "Ljava/util/TimeZone;", default.clone())
                .await?;
        }

        let cloned: ClassInstanceRef<Object> = jvm
            .invoke_virtual(&default, "java/lang/Object", "clone", "()Ljava/lang/Object;", ())
            .await?;
        Ok(ClassInstanceRef::new(cloned.instance))
    }

    async fn set_default(jvm: &Jvm, _: &mut RuntimeContext, timezone: ClassInstanceRef<Self>) -> Result<()> {
        tracing::debug!("java.util.TimeZone::setDefault({timezone:?})");

        let stored: ClassInstanceRef<Self> = if timezone.is_null() {
            Self::host_zone(jvm).await?
        } else {
            let cloned: ClassInstanceRef<Object> = jvm
                .invoke_virtual(&timezone, "java/lang/Object", "clone", "()Ljava/lang/Object;", ())
                .await?;
            ClassInstanceRef::new(cloned.instance)
        };
        jvm.put_static_field("java/util/TimeZone", "defaultTimeZone", "Ljava/util/TimeZone;", stored)
            .await
    }

    async fn get_available_ids(jvm: &Jvm, _: &mut RuntimeContext) -> Result<ClassInstanceRef<Array<String>>> {
        let mut ids: ClassInstanceRef<Array<String>> = jvm.instantiate_array("Ljava/lang/String;", ZONES.len()).await?.into();
        for (index, (id, _)) in ZONES.iter().enumerate() {
            let id = JavaLangString::from_rust_string(jvm, id).await?;
            jvm.store_array(&mut ids, index, [id]).await?;
        }
        Ok(ids)
    }

    async fn get_id(jvm: &Jvm, _: &mut RuntimeContext, this: ClassInstanceRef<Self>) -> Result<ClassInstanceRef<String>> {
        jvm.get_field(&this, "java/util/TimeZone", "ID", "Ljava/lang/String;").await
    }

    async fn set_id(jvm: &Jvm, _: &mut RuntimeContext, mut this: ClassInstanceRef<Self>, id: ClassInstanceRef<String>) -> Result<()> {
        tracing::debug!("java.util.TimeZone::setID({this:?}, {id:?})");

        if id.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "ID").await);
        }
        jvm.put_field(&mut this, "java/util/TimeZone", "ID", "Ljava/lang/String;", id).await
    }

    async fn clone(jvm: &Jvm, _: &mut RuntimeContext, this: ClassInstanceRef<Self>) -> Result<ClassInstanceRef<Object>> {
        jvm.invoke_special(&this, "java/lang/Object", "clone", "()Ljava/lang/Object;", ()).await
    }
}
