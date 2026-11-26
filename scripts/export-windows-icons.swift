import AppKit
import CoreGraphics
import Foundation

guard CommandLine.arguments.count == 4 else {
    fputs("Usage: export-windows-icons.swift <input-1024.png> <output.ico> <output-64.rgba>\n", stderr)
    exit(1)
}

let srcPath = CommandLine.arguments[1]
let icoPath = CommandLine.arguments[2]
let rgbaPath = CommandLine.arguments[3]

guard let srcImg = NSImage(contentsOfFile: srcPath),
      let cgSrc = srcImg.cgImage(forProposedRect: nil, context: nil, hints: nil) else {
    fputs("Failed to read \(srcPath)\n", stderr)
    exit(1)
}

let colorSpace = CGColorSpace(name: CGColorSpace.sRGB)!

func renderSize(_ dim: Int, premultiplied: Bool) -> (png: Data, rgba: Data)? {
    let alphaInfo: CGImageAlphaInfo = premultiplied ? .premultipliedLast : .last
    guard let ctx = CGContext(
        data: nil,
        width: dim,
        height: dim,
        bitsPerComponent: 8,
        bytesPerRow: dim * 4,
        space: colorSpace,
        bitmapInfo: alphaInfo.rawValue
    ) else {
        return nil
    }
    ctx.interpolationQuality = .high
    ctx.draw(cgSrc, in: CGRect(x: 0, y: 0, width: dim, height: dim))

    var rgba = Data()
    if let ptr = ctx.data {
        rgba = Data(bytes: ptr, count: dim * dim * 4)
    }

    guard let outCg = ctx.makeImage() else { return nil }
    let rep = NSBitmapImageRep(cgImage: outCg)
    guard let png = rep.representation(using: .png, properties: [:]) else { return nil }
    return (png, rgba)
}

// Build standard multi-resolution PNG-in-ICO (16, 24, 32, 48, 64, 128, 256)
let sizes = [16, 24, 32, 48, 64, 128, 256]
var pngs: [(Int, Data)] = []
for s in sizes {
    guard let rendered = renderSize(s, premultiplied: true) else {
        fputs("Failed to render \(s)x\(s)\n", stderr)
        exit(1)
    }
    pngs.append((s, rendered.png))
}

var ico = Data()
func appendU16(_ v: UInt16) {
    var le = v.littleEndian
    ico.append(Data(bytes: &le, count: 2))
}
func appendU32(_ v: UInt32) {
    var le = v.littleEndian
    ico.append(Data(bytes: &le, count: 4))
}

// ICONDIR header
appendU16(0) // reserved
appendU16(1) // type = ICO
appendU16(UInt16(pngs.count))

var offset: UInt32 = 6 + UInt32(pngs.count) * 16
for (dim, data) in pngs {
    ico.append(UInt8(dim >= 256 ? 0 : dim))
    ico.append(UInt8(dim >= 256 ? 0 : dim))
    ico.append(0) // color palette count
    ico.append(0) // reserved
    appendU16(1)  // color planes
    appendU16(32) // bits per pixel
    appendU32(UInt32(data.count))
    appendU32(offset)
    offset += UInt32(data.count)
}
for (_, data) in pngs {
    ico.append(data)
}

// Also export straight RGBA 64x64 for runtime egui::IconData (Windows/Linux titlebar & taskbar)
guard let ctx64 = CGContext(
    data: nil,
    width: 64,
    height: 64,
    bitsPerComponent: 8,
    bytesPerRow: 64 * 4,
    space: colorSpace,
    bitmapInfo: CGImageAlphaInfo.premultipliedLast.rawValue
) else {
    fputs("Failed to create 64x64 context\n", stderr)
    exit(1)
}
ctx64.interpolationQuality = .high
ctx64.draw(cgSrc, in: CGRect(x: 0, y: 0, width: 64, height: 64))
var straightRgba = Data(count: 64 * 64 * 4)
if let srcPtr = ctx64.data {
    let bytes = srcPtr.assumingMemoryBound(to: UInt8.self)
    straightRgba.withUnsafeMutableBytes { dstBuf in
        let dst = dstBuf.bindMemory(to: UInt8.self)
        for i in stride(from: 0, to: 64 * 64 * 4, by: 4) {
            let a = Int(bytes[i + 3])
            if a == 0 {
                dst[i] = 0; dst[i + 1] = 0; dst[i + 2] = 0; dst[i + 3] = 0
            } else if a == 255 {
                dst[i] = bytes[i]; dst[i + 1] = bytes[i + 1]; dst[i + 2] = bytes[i + 2]; dst[i + 3] = 255
            } else {
                dst[i] = UInt8(min(255, (Int(bytes[i]) * 255 + a / 2) / a))
                dst[i + 1] = UInt8(min(255, (Int(bytes[i + 1]) * 255 + a / 2) / a))
                dst[i + 2] = UInt8(min(255, (Int(bytes[i + 2]) * 255 + a / 2) / a))
                dst[i + 3] = UInt8(a)
            }
        }
    }
}

try ico.write(to: URL(fileURLWithPath: icoPath))
try straightRgba.write(to: URL(fileURLWithPath: rgbaPath))
print("Wrote \(icoPath) (\(ico.count) bytes, \(sizes.count) resolutions) and \(rgbaPath) (\(straightRgba.count) bytes)")
