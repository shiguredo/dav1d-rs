import argparse
import re
import subprocess
from datetime import date
from typing import Optional


# ファイルを読み込み、バージョンを更新
def update_version(file_path: str, dry_run: bool, release: bool) -> Optional[str]:
    with open(file_path, "r", encoding="utf-8") as f:
        content: str = f.read()

    # [package] セクション内のバージョンのみを取得
    package_section_match = re.search(
        r'\[package\].*?version\s*=\s*"([\d\.\w-]+)"', content, re.DOTALL
    )
    if not package_section_match:
        raise ValueError("Version not found in [package] section of Cargo.toml")

    current_version: str = package_section_match.group(1)

    # [package] セクションの開始位置を見つける
    package_start = content.find("[package]")
    # 次のセクション ([dependencies] など) の開始位置を見つける
    next_section = re.search(r"\n\[(?!package)", content[package_start:])
    if next_section:
        package_end = package_start + next_section.start()
        package_content = content[package_start:package_end]
    else:
        package_content = content[package_start:]

    # [package] セクション内のバージョンを更新
    if "-canary." in current_version and release:
        # canary 版を正式リリース版に変換 (2026.2.0-canary.2 -> 2026.2.0)
        updated_package, count = re.subn(
            r'(version\s*=\s*")(\d+\.\d+\.\d+)-canary\.\d+',
            lambda m: f"{m.group(1)}{m.group(2)}",
            package_content,
            count=1,  # 最初の1つだけを更新
        )
    elif "-canary." in current_version:
        # 次の canary にインクリメント (2026.2.0-canary.2 -> 2026.2.0-canary.3)
        updated_package, count = re.subn(
            r'(version\s*=\s*")(\d+\.\d+\.\d+-canary\.)(\d+)',
            lambda m: f"{m.group(1)}{m.group(2)}{int(m.group(3)) + 1}",
            package_content,
            count=1,  # 最初の1つだけを更新
        )
    else:
        # -canary.X がない場合、次のマイナーバージョンにして -canary.0 を追加
        updated_package, count = re.subn(
            r'(version\s*=\s*")(\d+)\.(\d+)\.(\d+)',
            lambda m: f"{m.group(1)}{m.group(2)}.{int(m.group(3)) + 1}.0-canary.0",
            package_content,
            count=1,  # 最初の1つだけを更新
        )

    if count == 0:
        raise ValueError("Version not found or incorrect format in [package] section")

    # 元のコンテンツの [package] セクション部分を更新後の内容に置き換える
    if next_section:
        new_content = content[:package_start] + updated_package + content[package_end:]
    else:
        new_content = content[:package_start] + updated_package

    # 新しいバージョンを確認 ([package] セクションから)
    new_package_version_match = re.search(
        r'\[package\].*?version\s*=\s*"([\d\.\w-]+)"', new_content, re.DOTALL
    )
    if not new_package_version_match:
        raise ValueError("Failed to extract the new version after the update.")

    new_version: str = new_package_version_match.group(1)

    print(f"Current version: {current_version}")
    print(f"New version: {new_version}")
    confirmation: str = (
        input("Do you want to update the version? (Y/n): ").strip().lower()
    )

    if confirmation != "y":
        print("Version update canceled.")
        return None

    # Dry-run 時の動作
    if dry_run:
        print("Dry-run: Version would be updated to:")
        print(new_content)
    else:
        with open(file_path, "w", encoding="utf-8") as f:
            f.write(new_content)
        print(f"Version updated in Cargo.toml to {new_version}")

    return new_version


# CHANGES.md の develop セクションをリリース版に更新
def update_changes(new_version: str, dry_run: bool) -> None:
    changes_path = "CHANGES.md"
    with open(changes_path, "r", encoding="utf-8") as f:
        content: str = f.read()

    release_date: str = date.today().isoformat()
    updated, count = re.subn(
        r"## develop",
        f"## {new_version}\n\n**リリース日**: {release_date}",
        content,
        count=1,
    )
    if count == 0:
        raise ValueError("## develop section not found in CHANGES.md")

    if dry_run:
        print("Dry-run: CHANGES.md would be updated to:")
        print(updated)
    else:
        with open(changes_path, "w", encoding="utf-8") as f:
            f.write(updated)
        print(f"CHANGES.md updated for release {new_version}")


# カレントブランチがリリース対象かどうかを確認する
def verify_release_branch() -> None:
    branch = subprocess.run(
        ["git", "rev-parse", "--abbrev-ref", "HEAD"],
        capture_output=True,
        text=True,
        check=True,
    ).stdout.strip()
    if not (branch == "develop" or branch.startswith("release/")):
        raise SystemExit(
            f"Release must be executed on develop or release/* branch (current: {branch})"
        )

    # 作業ツリーがクリーンであることを確認する
    status = subprocess.run(
        ["git", "status", "--porcelain"],
        capture_output=True,
        text=True,
        check=True,
    ).stdout.strip()
    if status:
        raise SystemExit("Working tree is not clean. Commit or stash changes first")


# cargo update shiguredo_dav1d を実行
def run_cargo_update(dry_run: bool) -> None:
    if dry_run:
        print("Dry-run: Would run 'cargo update shiguredo_dav1d'")
    else:
        subprocess.run(["cargo", "update", "shiguredo_dav1d"], check=True)
        print("cargo update shiguredo_dav1d executed")


# git コミットを実行
def git_commit_version(new_version: str, release: bool, dry_run: bool) -> None:
    if dry_run:
        print("Dry-run: Would run 'git add Cargo.toml Cargo.lock CHANGES.md'")
        print(
            f"Dry-run: Would commit '{'バージョンを' + new_version + ' に更新する'}'"
        )
    else:
        subprocess.run(["git", "add", "Cargo.toml", "Cargo.lock", "CHANGES.md"], check=True)
        if release:
            message = f"バージョンを {new_version} に更新する"
        else:
            message = f"canary バージョンを {new_version} に更新する"
        subprocess.run(["git", "commit", "-m", message], check=True)
        print(f"Version bumped and committed: {new_version}")


# git タグ、プッシュを実行
def git_operations_after_build(new_version: str, dry_run: bool) -> None:
    if dry_run:
        print(f"Dry-run: Would run 'git tag {new_version}'")
        print("Dry-run: Would run 'git push'")
        print(f"Dry-run: Would run 'git push origin {new_version}'")
    else:
        # リリース対象ブランチとクリーンな作業ツリーを確認してから実行する
        verify_release_branch()
        subprocess.run(["git", "tag", new_version], check=True)
        subprocess.run(["git", "push"], check=True)
        subprocess.run(["git", "push", "origin", new_version], check=True)


# メイン処理
def main() -> None:
    parser = argparse.ArgumentParser(
        description="Update Cargo.toml version and commit changes."
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Run in dry-run mode without making actual changes",
    )
    parser.add_argument(
        "--release",
        action="store_true",
        help=(
            "Convert the current canary version to a release version "
            "(e.g. 2026.2.0-canary.2 -> 2026.2.0) and update CHANGES.md"
        ),
    )
    args = parser.parse_args()

    cargo_toml_path: str = "Cargo.toml"

    # バージョン更新
    new_version: Optional[str] = update_version(cargo_toml_path, args.dry_run, args.release)

    if not new_version:
        return  # ユーザーが確認をキャンセルした場合、処理を中断

    # 正式リリースの場合は CHANGES.md の develop セクションを更新
    if args.release:
        update_changes(new_version, args.dry_run)

    # cargo update shiguredo_dav1d を実行
    run_cargo_update(args.dry_run)

    # バージョン更新後に git commit
    git_commit_version(new_version, args.release, args.dry_run)

    # git タグ付け、プッシュ
    git_operations_after_build(new_version, args.dry_run)


if __name__ == "__main__":
    main()
