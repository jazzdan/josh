extern crate git2;

extern crate tempdir;
use std::fs;

use super::*;
use std::path::Path;

fn to_ns(path: &str) -> String {
    return path.trim_matches('/').replace("/", "/refs/namespaces/");
}

pub fn fetch_refs_from_url(
    path: &Path,
    prefix: &str,
    url: &str,
    refs_prefix: &str,
    username: &str,
    password: &str,
) -> Result<(), git2::Error> {
    let spec = format!(
        "+{}:refs/namespaces/{}/{}",
        &refs_prefix,
        to_ns(prefix),
        &refs_prefix
    );

    let shell = shell::Shell {
        cwd: path.to_owned(),
    };
    let nurl = {
        let splitted: Vec<&str> = url.splitn(2, "://").collect();
        let proto = splitted[0];
        let rest = splitted[1];
        format!("{}://{}@{}", &proto, &username, &rest)
    };

    git2::Repository::open(path)
        .expect("no repo")
        .config()
        .unwrap()
        .set_str(
            "credential.helper",
            &format!("!f() {{ echo \"password={}\"; }}; f", &password),
        )?;

    let cmd = format!("git fetch {} '{}'", &nurl, &spec);
    println!("fetch_refs_from_url {:?} {:?} {:?}", cmd, path, "");

    /* shell.command(&"git prune"); */
    /* shell.command(&"git gc --auto"); */
    shell.command(&"git config gc.auto 0");

    let (_stdout, stderr) = shell.command(&cmd);
    if stderr.contains("fatal: Authentication failed") {
        return Err(git2::Error::from_str("auth"));
    }
    if stderr.contains("fatal:") {
        return Err(git2::Error::from_str("error"));
    }
    return Ok(());
}

pub fn push_head_url(
    repo: &git2::Repository,
    oid: git2::Oid,
    refname: &str,
    url: &str,
    username: &str,
    password: &str,
    namespace: &str,
) -> Result<String, ()> {
    let rn = format!("refs/{}", &namespace);

    let spec = format!("{}:{}", &rn, &refname);

    let shell = shell::Shell {
        cwd: repo.path().to_owned(),
    };
    let nurl = {
        let splitted: Vec<&str> = url.splitn(2, "://").collect();
        let proto = splitted[0];
        let rest = splitted[1];
        format!("{}://{}@{}", &proto, &username, &rest)
    };
    some_or!(
        repo.config()
            .unwrap()
            .set_str(
                "credential.helper",
                &format!("!f() {{ echo \"password={}\"; }}; f", &password),
            )
            .ok(),
        {
            return Err(());
        }
    );
    let cmd = format!("git push {} '{}'", &nurl, &spec);
    let mut fakehead = repo
        .reference(&rn, oid, true, "push_head_url")
        .expect("can't create reference");
    let (stdout, stderr) = shell.command(&cmd);
    fakehead.delete().expect("fakehead.delete failed");
    println!("{}", &stderr);
    println!("{}", &stdout);

    let stderr = stderr.replace(&rn, "JOSH_PUSH");

    return Ok(stderr);
}

pub fn create_local(path: &Path) {
    println!("init base repo: {:?}", path);
    fs::create_dir_all(path).expect("can't create_dir_all");

    match git2::Repository::open(path) {
        Ok(_) => {
            println!("repo exists");
            return;
        }
        Err(_) => {}
    };

    match git2::Repository::init_bare(path) {
        Ok(_) => {
            println!("repo initialized");
            return;
        }
        Err(_) => {}
    }
}
