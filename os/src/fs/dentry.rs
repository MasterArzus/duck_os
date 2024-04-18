//! dentry模块
//! 

/*
    所有的dentry构成一棵树的形式。并且应该存放在cache中。
    其实没有什么功能，就是方便查找路径。
    1. 数据结构
        1）name: 目录项名称（短名）
        2）inode：相关联的 inode
        3）path：路径名 
        4）parent: 父目录项
        5）children：子目录项
    
    2. 功能
        1） 生成 hash值，方便放入hash table中（待定）
*/


use core::any::Any;

use alloc::{string::{String, ToString}, sync::{Arc, Weak}, vec::Vec};
use hashbrown::HashMap;
use lazy_static::lazy_static;

use crate::sync::SpinLock;

use super::{file_system::FILE_SYSTEM_MANAGER, info::{InodeMode, OpenFlags}, inode::Inode};

// TODO: 很多细节：诸如 函数正确性和逻辑、函数假设是否满足、path是否规范、Option、锁之类的都没考虑完全。

pub struct DentryMeta {
    pub inner: SpinLock<DentryMetaInner>,
}

// 这些数据都是可能会被修改的，所以用锁保护起来。
pub struct DentryMetaInner {
    pub d_name: String,
    pub d_path: String,
    pub d_inode: Arc<dyn Inode>,
    pub d_parent: Option<Weak<dyn Dentry>>,
    pub d_child: Vec<Arc<dyn Dentry>>,
}

impl DentryMeta {
    pub fn new(
        name: String,
        path: String,
        inode: Arc<dyn Inode>,
        parent: Option<Arc<dyn Dentry>>,
        child: Vec<Arc<dyn Dentry>>,
    ) -> Self {
        let name = format(&name);
        let parent = match parent {
            Some(parent) => Some(Arc::downgrade(&parent)),
            None => None,
        };
        Self { 
            inner: SpinLock::new(
                DentryMetaInner {
                    d_name: name,
                    d_path: path,
                    d_inode: inode,
                    d_parent: parent,
                    d_child: child
                }
            )
        }
    }
}

pub trait Dentry: Sync + Send + Any {
    fn metadata(&self) -> &DentryMeta;
    fn load_child(&self, this: Arc<dyn Dentry>);
    fn load_all_child(&self, this: Arc<dyn Dentry>);
    // 查找inode的子节点，并返回对应的dentry,负责把child_inode创建好，挂在dentry上
    // 从磁盘上找相关的数据，然后建立对应的dentry,并返回
    fn open(&self, this: Arc<dyn Dentry>, name: &str, flags: OpenFlags) -> Option<Arc<dyn Dentry>>;
    fn mkdir(&self, path: &str, mode: InodeMode) -> Arc<dyn Dentry>;
    fn mknod(&self, path: &str, mode: InodeMode, dev_id: Option<usize>) -> Arc<dyn Dentry>;
    fn unlink(&self, child: Arc<dyn Dentry>);
}

lazy_static! {
    // (path, Dentry)
    pub static ref DENTRY_CACHE: SpinLock<HashMap<String, Arc<dyn Dentry>>> = SpinLock::new(HashMap::new());
}

// Assumption: 此时的path是合法的绝对路径，并且是format的
// function: 在Cache中查找dentry
pub fn path_to_dentry_cache(path: &str) -> Option<Arc<dyn Dentry>> {
    if let Some(dentry) = DENTRY_CACHE
        .lock()
        .get(path) {
            Some(Arc::clone(dentry))
        } else {
            None
        }
}
// Assumption: 此时的path是合法的绝对路径，并且是format的
// return：路径对应的dentry 相关的dentry已经在树上和cache中了
// TODO: 这里的meta反复的上锁，会不会出现问题？还是直接得到一个上了锁的，再统一修改。
/*
    1. openat：这个函数用来查找父dentry，此时的父dentry一定在cache中或者在树上。所以一定可以找到。
*/
pub fn path_to_dentry(path: &str) -> Option<Arc<dyn Dentry>> {
    // 绝对路径在cache中查找
    if let Some(dentry) = path_to_dentry_cache(path){
        Some(Arc::clone(&dentry))
    } else {
        // 没找到，匹配前最大子串
        // TODO：优化部分，这里可以匹配一下cache中的最大匹配前字串，从而减少查找的时间
        
        // 如果找到了前最大子串，则pa_dentry不是从根开始
        // 否则从根开始
        let mut pa_dentry = FILE_SYSTEM_MANAGER.root_dentry();
        let mut pa_inode: Arc<dyn Inode>;
        // if let Some(inode) = &pa_dentry.metadata().inner.lock().d_inode {
        //     pa_inode = inode.clone();
        // } else {
        //     todo!();
        // }
        pa_inode = pa_dentry.metadata().inner.lock().d_inode.clone();
        let path_vec: Vec<&str> = path
            .split('/')
            .filter(|name| *name != "" )
            .collect();
        // let end_name = path_vec[path_vec.len() - 1];
        // 开始遍历所有的路径
        for name in path_vec.into_iter() {
            // 先从树上找
            if let Some(dentry) = pa_dentry
                .clone()
                .metadata()
                .inner
                .lock()
                .d_child
                .iter()
                .find(|d| d.metadata().inner.lock().d_name == name ) {
                    // 在树上找到了，插入cache中，然后继续遍历
                    DENTRY_CACHE.lock().insert(dentry.metadata().inner.lock().d_path.to_string(), dentry.clone());
                    pa_dentry = dentry.clone();
                    // pa_inode = pa_dentry.metadata().inner.lock().d_inode.as_ref().clone();
                    // TODO： 如果对应的dentry没有inode，该怎么办？？？
                    // if let Some(inode) = &pa_dentry.metadata().inner.lock().d_inode {
                    //     pa_inode = inode.clone();
                    // } else {
                    //     todo!();
                    // }
                    pa_inode = pa_dentry.metadata().inner.lock().d_inode.clone();
                    continue;
                }
            else {
                // 树上没有找到，直接返回错误
                // 因为我确保了磁盘上每一个dentry都在树上 ----> 1. 初始化时所有的都在树上 2.创建新的dentry也给挂在树上
                todo!();
                
                // // 这里看上去是查找，同时也做了创建的工作。
                // if let Some(child_dentry) = pa_dentry.look_up(Arc::clone(&pa_dentry), name) {
                //     // 磁盘上找到了，插入树中 和 cache中，继续遍历
                //     DENTRY_CACHE.lock().insert(child_dentry.metadata().inner.lock().d_path.to_string(), child_dentry.clone());
                //     pa_dentry.metadata().inner.lock().d_child.push(child_dentry.clone());
                //     child_dentry.metadata().inner.lock().d_parent = Some(Arc::downgrade(&pa_dentry));
                //     pa_dentry = child_dentry.clone();
                //     // if let Some(inode) = &pa_dentry.metadata().inner.lock().d_inode {
                //     //     pa_inode = inode.clone();
                //     // } else {
                //     //     todo!();
                //     // }
                //     pa_inode = pa_dentry.metadata().inner.lock().d_inode.clone();
                // } else {
                //     // 磁盘上也没有,不可能会没有，所以要报错！
                //     todo!()
                // }
            }
        }
        Some(pa_dentry)
    }
}

// 得到路径上最后一个name
pub fn dentry_name(path: &str) -> &str {
    if path == "" {
        return "";
    }
    let names: Vec<&str> = path.split('/').filter(|name| *name != "").collect();
    names[names.len() - 1]
}

pub fn path_plus_name(path: &str, name: &str) -> String {
    let mut final_path: String = path.to_string();
    if !path.ends_with("/") {
        final_path.push('/');
    }
    final_path.push_str(name);
    final_path
}

// Assumption: path 只为 / 开头的路径 或者 ../ ./ 开头的合法路径
pub fn is_relative_path(path: &str) -> bool {
    if path.starts_with("/") {
        false
    } else {
        true
    }
}

// Assumption: 此时的 path 已经是合法的绝对路径，同时也是format路径，找到它的父路径
// 返回的形式 / or /xxx
pub fn parent_path(path: &str) -> String {
    if path == "/" {
        return "/".to_string();
    } else {
        let mut names: Vec<&str> = path.split("/").filter(|name| *name != "" ).collect();
        if names.len() == 1 {
            return "/".to_string();
        } else {
            names.insert(0, "");
            names.pop();
            names.join("/")
        }
    }
}

// Assumption: 此时的 path 是合法的路径： ../ 和 ./
// 去除掉 ../ 和 ./
pub fn format_rel_path(path: &str) -> String {
    path.trim_start_matches(|c| c == '.' || c == '/').to_string()
} 

// 相对路径 ---> 绝对路径 暂时先命名为dirfd_and_path
// Assumption： path已经是相对路径 ../ 或者 ./
// 返回形式: /xxx/yy 或者 / 
pub fn dirfd_and_path(_dirfd: usize, path: &str) -> String {
    let format_path = format(path);
    // TODO: 通过 current_process 得到对应的fd_table，然后找到dirfd 文件，目录项，path
    // 暂时先使用 format_dirfd
    let format_dirfd = format("/path");
    // path is ../
    if format_path.starts_with("../") {
        let mut cwd = parent_path(&format_dirfd);
        let end_path = format_rel_path(&format_path);
        if !cwd.ends_with('/') {
            cwd.push('/');
        }
        cwd.push_str(&end_path);
        cwd
    } else if format_path.starts_with("./"){
        // path is ./
        let mut cwd = format_dirfd;
        let end_path = format_rel_path(&format_path);
        if !cwd.ends_with('/') {
            cwd.push('/');
        }
        cwd.push_str(&end_path);
        cwd
    } else {
        todo!()
    }
}

// 相对路径 ---> 绝对路径 由cwd -> abs_path
// Assumption： path已经是相对路径 ../ 或者 ./ 且 cwd已经是 / or /xxx/yyy
// 返回形式: /xxx/yy 或者 / 
pub fn cwd_and_path(path: &str, cwd: &str) -> String {
    if path.starts_with("../") {
        let mut cwd_pa = parent_path(cwd);
        let f_path = format_rel_path(path);
        if !&cwd_pa.ends_with("/") {
            cwd_pa.push('/');
        }
        cwd_pa.push_str(&f_path);
        cwd_pa
    } else if path.starts_with("./") {
        let mut cwd: String = cwd.to_string();
        let f_path = format_rel_path(path);
        if !&cwd.ends_with("/") {
            cwd.push('/');
        }
        cwd.push_str(&f_path);
        cwd
    } else {
        todo!();
    }
}

// 规范化路径，主要是去掉 \t \n等，还有最后一个 “/”
pub fn format(path: &str) -> String {
    if path == "" {
        return "".to_string();
    } else {
        path.trim_end_matches(|c| c == '/' || c == '\t' || c == '\n').to_string()
    }
}