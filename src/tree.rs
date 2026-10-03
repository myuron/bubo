use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeKind {
    Dir { children: Vec<Node>, expanded: bool },
    File,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    pub name: String,
    pub path: PathBuf,
    pub kind: NodeKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisibleItem {
    pub depth: usize,
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub expanded: bool,
}

/// `root` 配下を再帰的に走査し、`.md` ファイルとそれを含むディレクトリのみからなるツリーを構築する。
/// シンボリックリンクのディレクトリは辿らない。読み取れないエントリやサブディレクトリは無視する。
pub fn build_tree(root: &Path) -> io::Result<Vec<Node>> {
    let mut dirs = Vec::new();
    let mut files = Vec::new();
    for entry in std::fs::read_dir(root)? {
        let Ok(entry) = entry else {
            continue;
        };
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        if file_type.is_dir() {
            let Ok(children) = build_tree(&path) else {
                continue;
            };
            if !children.is_empty() {
                dirs.push(Node {
                    name,
                    path,
                    kind: NodeKind::Dir {
                        children,
                        expanded: false,
                    },
                });
            }
        } else if is_markdown(&path) && path.is_file() {
            files.push(Node {
                name,
                path,
                kind: NodeKind::File,
            });
        }
    }
    dirs.sort_by(|a, b| a.name.cmp(&b.name));
    files.sort_by(|a, b| a.name.cmp(&b.name));
    dirs.extend(files);
    Ok(dirs)
}

fn is_markdown(path: &Path) -> bool {
    path.extension().is_some_and(|ext| ext == "md")
}

/// 展開状態に従ってツリーを平坦化し、画面に表示される項目の一覧を返す。
pub fn visible(nodes: &[Node]) -> Vec<VisibleItem> {
    let mut items = Vec::new();
    collect_visible(nodes, 0, &mut items);
    items
}

fn collect_visible(nodes: &[Node], depth: usize, items: &mut Vec<VisibleItem>) {
    for node in nodes {
        let (is_dir, expanded) = match &node.kind {
            NodeKind::Dir { expanded, .. } => (true, *expanded),
            NodeKind::File => (false, false),
        };
        items.push(VisibleItem {
            depth,
            name: node.name.clone(),
            path: node.path.clone(),
            is_dir,
            expanded,
        });
        if let NodeKind::Dir {
            children,
            expanded: true,
        } = &node.kind
        {
            collect_visible(children, depth + 1, items);
        }
    }
}

/// 可視リスト上の `index` 番目がディレクトリであれば展開状態を反転する。
pub fn toggle(nodes: &mut [Node], index: usize) {
    let mut remaining = index;
    toggle_at(nodes, &mut remaining);
}

/// 可視順に `remaining` を減らしながら探索し、0 に達したノードをトグルする。見つかれば true。
fn toggle_at(nodes: &mut [Node], remaining: &mut usize) -> bool {
    for node in nodes {
        if *remaining == 0 {
            if let NodeKind::Dir { expanded, .. } = &mut node.kind {
                *expanded = !*expanded;
            }
            return true;
        }
        *remaining -= 1;
        if let NodeKind::Dir {
            children,
            expanded: true,
        } = &mut node.kind
            && toggle_at(children, remaining)
        {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn setup(files: &[&str]) -> TempDir {
        let dir = tempfile::tempdir().unwrap();
        for f in files {
            let path = dir.path().join(f);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, "content").unwrap();
        }
        dir
    }

    fn names(items: &[VisibleItem]) -> Vec<&str> {
        items.iter().map(|i| i.name.as_str()).collect()
    }

    #[test]
    fn build_tree_includes_only_markdown_files() {
        let dir = setup(&["a.md", "b.txt", "c.rs"]);
        let tree = build_tree(dir.path()).unwrap();
        assert_eq!(names(&visible(&tree)), vec!["a.md"]);
    }

    #[test]
    fn build_tree_prunes_directories_without_markdown() {
        let dir = setup(&["docs/guide.md", "src/main.rs", "empty_parent/inner/x.txt"]);
        fs::create_dir(dir.path().join("empty")).unwrap();
        let tree = build_tree(dir.path()).unwrap();
        assert_eq!(names(&visible(&tree)), vec!["docs"]);
    }

    #[test]
    fn build_tree_includes_hidden_markdown() {
        let dir = setup(&[".hidden.md", ".config/notes.md"]);
        let tree = build_tree(dir.path()).unwrap();
        assert_eq!(names(&visible(&tree)), vec![".config", ".hidden.md"]);
    }

    #[test]
    fn build_tree_sorts_dirs_first_then_by_name() {
        let dir = setup(&["z.md", "a.md", "b/x.md", "a_dir/y.md"]);
        let tree = build_tree(dir.path()).unwrap();
        assert_eq!(names(&visible(&tree)), vec!["a_dir", "b", "a.md", "z.md"]);
    }

    #[test]
    fn build_tree_collapses_directories_initially() {
        let dir = setup(&["docs/deep/a.md"]);
        let tree = build_tree(dir.path()).unwrap();
        let items = visible(&tree);
        assert_eq!(items.len(), 1);
        assert!(items[0].is_dir);
        assert!(!items[0].expanded);
    }

    #[test]
    fn toggle_expands_and_collapses_directory() {
        let dir = setup(&["docs/deep/a.md", "docs/b.md", "top.md"]);
        let mut tree = build_tree(dir.path()).unwrap();

        toggle(&mut tree, 0);
        let items = visible(&tree);
        assert_eq!(names(&items), vec!["docs", "deep", "b.md", "top.md"]);
        assert_eq!(items[1].depth, 1);
        assert!(items[0].expanded);

        toggle(&mut tree, 1);
        let items = visible(&tree);
        assert_eq!(
            names(&items),
            vec!["docs", "deep", "a.md", "b.md", "top.md"]
        );
        assert_eq!(items[2].depth, 2);
        assert_eq!(items[2].path, dir.path().join("docs/deep/a.md"));

        toggle(&mut tree, 0);
        assert_eq!(names(&visible(&tree)), vec!["docs", "top.md"]);
    }

    #[test]
    fn toggle_on_file_or_out_of_range_does_nothing() {
        let dir = setup(&["a.md"]);
        let mut tree = build_tree(dir.path()).unwrap();
        let before = tree.clone();
        toggle(&mut tree, 0);
        toggle(&mut tree, 5);
        assert_eq!(tree, before);
    }

    #[cfg(unix)]
    #[test]
    fn build_tree_does_not_follow_directory_symlinks() {
        let dir = setup(&["docs/a.md"]);
        std::os::unix::fs::symlink(dir.path(), dir.path().join("docs/loop")).unwrap();
        let mut tree = build_tree(dir.path()).unwrap();
        toggle(&mut tree, 0);
        assert_eq!(names(&visible(&tree)), vec!["docs", "a.md"]);
    }
}
