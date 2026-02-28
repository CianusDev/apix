  git add Cargo.toml Cargo.lock
  git commit -m "fix: bump version to 1.0.1"
  git push origin main

  # 3. Tagger → déclenche le build + release
  git tag v1.0.1
  git push origin v1.0.1
