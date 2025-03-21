set -ex

if [ -z "$(git status --porcelain)" ]; then 
  # Working directory clean
  
  (
    # dx fmt -s
    cargo fmt
  )

    git add .
    git status
    git commit -m "cargo fmt $(date)" || true

    (  cargo fix   --allow-dirty || true   )
    
    git add .
    git status
    git commit -m "cargo fix $(date)" || true
    git push

else 
    git status
    set +x
    echo
    echo "!!!"
    echo
    echo "WORKING DIRECTORY NOT CLEAN"
    echo "PLZ COMMIT CHANGES"
    exit 66
fi