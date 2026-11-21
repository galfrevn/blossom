import { expect, test } from "bun:test";
import { crc32, inflateSync } from "node:zlib";
import { encodePng } from "../png";

const signatureBytes = 8;
const lengthBytes = 4;
const typeBytes = 4;
const checksumBytes = 4;
const headerDepthOffset = 8;
const eightBitDepth = 8;
const colorTypeWithAlpha = 6;
const noFilter = 0;
const twoPixels = 2;
const bytesPerRowOfTwo = 8;
const channelsPerPixel = 4;
const grayBytes = Buffer.from(
  "c8".repeat(twoPixels * twoPixels * channelsPerPixel),
  "hex",
);
const fourPixels = Buffer.from("ff0000ff0000ff800909090900000000", "hex");

type Chunk = {
  type: string;
  data: Uint8Array;
  checksum: number;
  checkedBytes: Uint8Array;
};

function readChunks(png: Uint8Array): Chunk[] {
  const view = new DataView(png.buffer, png.byteOffset, png.byteLength);
  const chunks: Chunk[] = [];
  let position = signatureBytes;
  while (position < png.length) {
    const length = view.getUint32(position);
    const typeStart = position + lengthBytes;
    const dataStart = typeStart + typeBytes;
    chunks.push({
      type: new TextDecoder().decode(png.subarray(typeStart, dataStart)),
      data: png.subarray(dataStart, dataStart + length),
      checksum: view.getUint32(dataStart + length),
      checkedBytes: png.subarray(typeStart, dataStart + length),
    });
    position = dataStart + length + checksumBytes;
  }
  return chunks;
}

test("a PNG starts with the PNG signature and holds a header, the pixels and an end", () => {
  const png = encodePng(
    1,
    new Uint8Array(grayBytes.subarray(0, channelsPerPixel)),
  );

  expect(Buffer.from(png.subarray(0, signatureBytes)).toString("hex")).toBe(
    "89504e470d0a1a0a",
  );
  expect(readChunks(png).map((chunk) => chunk.type)).toEqual([
    "IHDR",
    "IDAT",
    "IEND",
  ]);
});

test("every chunk carries the CRC-32 of its type and data", () => {
  const png = encodePng(twoPixels, new Uint8Array(grayBytes));

  for (const chunk of readChunks(png)) {
    expect(chunk.checksum).toBe(crc32(chunk.checkedBytes));
  }
});

test("the header says the size and eight bit RGBA, and the pixels inflate back row by row", () => {
  const [header, data] = readChunks(
    encodePng(twoPixels, new Uint8Array(fourPixels)),
  );
  if (header === undefined || data === undefined) {
    throw new Error("the PNG has no header or no pixel data");
  }
  const headerView = new DataView(
    header.data.buffer,
    header.data.byteOffset,
    header.data.byteLength,
  );

  expect([headerView.getUint32(0), headerView.getUint32(lengthBytes)]).toEqual([
    twoPixels,
    twoPixels,
  ]);
  expect([...header.data.subarray(headerDepthOffset)]).toEqual([
    eightBitDepth,
    colorTypeWithAlpha,
    0,
    0,
    0,
  ]);
  expect([...inflateSync(data.data)]).toEqual([
    noFilter,
    ...fourPixels.subarray(0, bytesPerRowOfTwo),
    noFilter,
    ...fourPixels.subarray(bytesPerRowOfTwo),
  ]);
});
