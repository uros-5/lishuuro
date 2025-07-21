
# minimize tailwind classes

# get tailwind file
npm run build
tailwindfile=$(fd --search-path=dist "tailwind.*\.css" --exec echo)	
output="not-tailwind"
cp -r ${tailwindfile} src/assets/tailwind.css

# run not-tailwind
not-tailwind --build-ts-map --run vue html ts --ignored ./public/board/chessground.css --output ${output} 
# replace templates
rm -rf src/components/*
cp -r ${output}/src/components/* src/components
cp -r ${output}/src/not-tailwind.ts src/not-tailwind.ts

echo > src/assets/tailwind.css
cat ${output}/src/assets/tailwind.css > src/assets/tailwind.css

# rebuild again
npm run build

# clean
mv dist/src/components/index.html dist
rm -rf dist/src
rm -rf src
rm -rf ${output}
rm -rf src/assets/tailwind.css
git checkout HEAD -- src
rm -rf ../assets
cp -r dist ..
mv ../dist ../assets
mv ../assets/index.html ../assets/index.j2
