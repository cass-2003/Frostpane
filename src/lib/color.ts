export function extractDominantColor(base64Png: string): Promise<string> {
  return new Promise((resolve) => {
    const img = new Image();
    img.onload = () => {
      const canvas = document.createElement("canvas");
      const size = 32;
      canvas.width = size;
      canvas.height = size;
      const ctx = canvas.getContext("2d");
      if (!ctx) { resolve("rgba(28,32,48,0.42)"); return; }

      ctx.drawImage(img, 0, 0, size, size);
      const data = ctx.getImageData(0, 0, size, size).data;

      let r = 0, g = 0, b = 0, count = 0;
      for (let i = 0; i < data.length; i += 16) {
        const a = data[i + 3];
        if (a < 50) continue;
        r += data[i];
        g += data[i + 1];
        b += data[i + 2];
        count++;
      }

      if (count === 0) { resolve("rgba(28,32,48,0.42)"); return; }

      r = Math.round(r / count);
      g = Math.round(g / count);
      b = Math.round(b / count);

      resolve(`rgba(${r},${g},${b},0.25)`);
    };
    img.onerror = () => resolve("rgba(28,32,48,0.42)");
    img.src = `data:image/png;base64,${base64Png}`;
  });
}

export async function computeFenceDominantColor(iconDataList: string[]): Promise<string> {
  const colors = await Promise.all(
    iconDataList.filter(Boolean).slice(0, 6).map(extractDominantColor)
  );

  if (colors.length === 0) return "rgba(28,32,48,0.42)";

  let r = 0, g = 0, b = 0;
  for (const c of colors) {
    const m = c.match(/rgba?\((\d+),(\d+),(\d+)/);
    if (m) { r += +m[1]; g += +m[2]; b += +m[3]; }
  }
  r = Math.round(r / colors.length);
  g = Math.round(g / colors.length);
  b = Math.round(b / colors.length);

  return `rgba(${r},${g},${b},0.3)`;
}
