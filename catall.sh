echo "PROJECT: "; 
pwd;
echo; echo; 
echo "--- PROJECT FILE LIST ---"; 

git ls-files

echo "--- END PROJECT FILE LIST ---"; 
echo; echo; 

echo "--- LAST COMMITS ---"; 
echo; echo; 

git log -5


echo; echo; 
echo "--- RUST SOURCE CODE ---"; 

for f in $(git ls-files -- "*.rs" "*.yaml" "*.md" "*.sh" "*.toml" "*.html" "*.wgsl"); do 

    echo "--- BEGIN FILE $f ---"; 
    echo; echo; 
    echo "// $f";
    echo; echo; 
    cat $f; 
    echo; echo; 
    echo "// end of: $f";
    echo; echo; 
    echo "--- END FILE $f ---"; 
    echo; echo; 

done