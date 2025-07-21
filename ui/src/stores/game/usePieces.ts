import { defineStore } from "pinia";
import { createZipFile, extractZipFile, IndexedDBStorage } from '@/helpers/zip'
import { ref } from "vue";

export const usePieces = defineStore('pieces', () => {

  const pieces = ref([
    { value: 'merida', url: '/pieces/merida/wN.svg' },
    { value: 'maestro', url: '/pieces/maestro/wN.svg' },
    { value: 'kaneo', url: '/pieces/kaneo/wN.svg' },
    { value: 'custom', url: '/pieces/custom.svg' },
  ]);
  const db = new IndexedDBStorage("lishuuro", "zip")
  let cssFile = ""
  function cssSelector(style: string, piece: string, color: string, url: string) {
    return `
      .cg-wrap[data-piece$="${style}"] piece.${piece.toLowerCase()}-piece.${color} {
        background-image: url(${url});
      }
      `
  }
  type Pieces = Blob
  type StyleName = string
  const pieceBlobs: Record<StyleName, Record<string, Pieces>> = {};

  let fileNames = [
    'A',
    'B',
    'C',
    'G',
    'K',
    'N',
    'P',
    'Q',
    'R',
  ]
  async function loadPieces() {
    if (document.querySelector("style#custom-piece-set") != undefined) return

    let css = document.createElement("style")
    let toAddCss = false;
    css.setAttribute("id", "custom-piece-set")
    let zips = await db.getAllZips()
    if (!zips) return
    let resolver = (value: unknown) => {};
    let wait = new Promise((resolve, reject) => {
      resolver = resolve;
    })
    let zipLen = zips.length

    zips.forEach(async (zipFile, index) => {
      // @ts-ignore
      let contents = await extractZipFile(zipFile.blob);
      pieceBlobs[zipFile.name] = contents
      let introUrl = "/pieces/merida/wN.svg";
      let toAddPiece = false;

      for (const [relativePath, file] of Object.entries(contents)) {
        if (relativePath.length < 2) continue
        if (!fileNames.includes(relativePath.at(1)!)) continue
        const color = relativePath.at(0) == "w" ? "white" : "black";
        const piece = relativePath.at(1)!
        let url = URL.createObjectURL(file)
        if (relativePath.endsWith(".svg")) {
          let content = await file.text()
          url = `data:image/svg+xml;base64,${btoa(content)} `
        }
        if (relativePath.startsWith("wN")) {
          introUrl = url;
        }
        cssFile += cssSelector(zipFile.name, piece, color, url) + "\n"
        toAddPiece = true;
        toAddCss = true;
      }
      if (toAddPiece) {
        const custom = pieces.value.pop()!;
        const newStyle = { value: zipFile.name, url: introUrl }
        pieces.value.push(newStyle)
        pieces.value.push(custom)
      }
      if(index == zipLen - 1) {
        resolver(undefined)
      }
    })
    await wait;
    if (toAddCss) {
      css.textContent = cssFile;
      css.setAttribute("id", "custom-piece-set")
      document.querySelector("head")!.appendChild(css)
    }
  }

  return { pieces, loadPieces }
});
