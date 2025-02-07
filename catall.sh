      
# This script bundles the project's source code into a single text file
# for easy input to AI models. Run: bash catall.sh > output_file.txt
    
# This program is intended to be run as this:
# $ bash catall.sh >~/Downloads/project_name.txt
#
# The purpose of this is to easily bundle the project into a single TXT file
# that any AI can read it, so that we can get help from them and make them
# code this.

echo "PROJECT: "; 
pwd;
echo; echo; 
echo "--- PROJECT FILE LIST ---"; 
echo; echo; 

git ls-files

echo; echo; 
echo "--- END PROJECT FILE LIST ---"; 
echo; echo; 

echo; echo; 
echo "--- LAST COMMITS ---"; 
echo; echo; 

git log -5


echo; echo; 
echo "--- PROJECT SOURCE CODE ---"; 
echo; echo; 
echo; echo; 

# for f in $(git ls-files -- "*.rs" "*.yaml" "*.md" "*.sh" "*.toml" "*.html" "*.wgsl"); do 
for f in $(git ls-files -- "*.rs" "*.toml" "*.md"); do 

    echo "--- BEGIN FILE \`$f\` ---"; 
    echo; echo; 
    # echo "// $f";
    # echo; echo; 
    cat $f; 
    echo; echo; 
    # echo "// end of: $f";
    # echo; echo; 
    echo "--- END FILE \`$f\` ---"; 
    echo; echo; 

done