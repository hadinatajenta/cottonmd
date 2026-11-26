import AppKit
import CoreGraphics
import Foundation

guard CommandLine.arguments.count == 3 else {
    fputs("Usage: render-liquid-glass-icon.swift <input-image> <output-png>\n", stderr)
    exit(1)
}

let inputPath = CommandLine.arguments[1]
let outputPath = CommandLine.arguments[2]

guard let srcImage = NSImage(contentsOfFile: inputPath),
      let cgSrc = srcImage.cgImage(forProposedRect: nil, context: nil, hints: nil) else {
    fputs("Failed to load input image: \(inputPath)\n", stderr)
    exit(1)
}

let size = 1024
let canvasSize = CGFloat(size)
let colorSpace = CGColorSpace(name: CGColorSpace.sRGB)!

guard let ctx = CGContext(
    data: nil,
    width: size,
    height: size,
    bitsPerComponent: 8,
    bytesPerRow: size * 4,
    space: colorSpace,
    bitmapInfo: CGImageAlphaInfo.premultipliedLast.rawValue
) else {
    fputs("Failed to create CGContext\n", stderr)
    exit(1)
}

ctx.interpolationQuality = .high
ctx.setAllowsAntialiasing(true)
ctx.setShouldAntialias(true)

// Apple HIG macOS squircle: 824x824 tile centered on 1024x1024 canvas
let tileInset: CGFloat = 100.0
let tileSize: CGFloat = canvasSize - tileInset * 2.0
let tileRect = CGRect(x: tileInset, y: tileInset, width: tileSize, height: tileSize)

// Superellipse squircle path (n = 4.6 closely matches Apple continuous-curvature icon shape)
func makeSquirclePath(in rect: CGRect, inset: CGFloat = 0.0) -> CGPath {
    let r = rect.insetBy(dx: inset, dy: inset)
    let cx = r.midX
    let cy = r.midY
    let rx = r.width * 0.5
    let ry = r.height * 0.5
    let n: Double = 4.65
    let exp = 2.0 / n
    let steps = 512

    let path = CGMutablePath()
    for i in 0...steps {
        let theta = (Double(i) / Double(steps)) * (2.0 * Double.pi)
        let ct = cos(theta)
        let st = sin(theta)
        let x = cx + rx * CGFloat(copysign(pow(abs(ct), exp), ct))
        let y = cy + ry * CGFloat(copysign(pow(abs(st), exp), st))
        if i == 0 {
            path.move(to: CGPoint(x: x, y: y))
        } else {
            path.addLine(to: CGPoint(x: x, y: y))
        }
    }
    path.closeSubpath()
    return path
}

let squircle = makeSquirclePath(in: tileRect)

// 1. Soft macOS Dock ambient + drop shadow
ctx.saveGState()
ctx.setShadow(
    offset: CGSize(width: 0, height: -14),
    blur: 28,
    color: CGColor(srgbRed: 0, green: 0, blue: 0, alpha: 0.44)
)
ctx.addPath(squircle)
ctx.setFillColor(CGColor(srgbRed: 0.08, green: 0.09, blue: 0.12, alpha: 1.0))
ctx.fillPath()
ctx.restoreGState()

// 2. Clipped full-bleed artwork inside the Apple squircle
ctx.saveGState()
ctx.addPath(squircle)
ctx.clip()

// Subtle 1.06x crop so the crystal sits generously inside the 824x824 tile
let scale: CGFloat = 1.06
let drawW = tileRect.width * scale
let drawH = tileRect.height * scale
let drawRect = CGRect(
    x: tileRect.midX - drawW * 0.5,
    y: tileRect.midY - drawH * 0.5,
    width: drawW,
    height: drawH
)
ctx.draw(cgSrc, in: drawRect)

// 3. Delicate Liquid Glass top-left overhead optical sheen
let sheenColors = [
    CGColor(srgbRed: 0.90, green: 0.95, blue: 1.0, alpha: 0.12),
    CGColor(srgbRed: 0.72, green: 0.84, blue: 0.98, alpha: 0.03),
    CGColor(srgbRed: 0.0, green: 0.0, blue: 0.0, alpha: 0.0),
] as CFArray
if let sheenGrad = CGGradient(colorsSpace: colorSpace, colors: sheenColors, locations: [0.0, 0.36, 0.72]) {
    ctx.drawLinearGradient(
        sheenGrad,
        start: CGPoint(x: tileRect.minX + 60, y: tileRect.maxY),
        end: CGPoint(x: tileRect.maxX - 80, y: tileRect.minY + 120),
        options: []
    )
}

// 4. Liquid Glass luminous refractive inner bevel (top-left bright frost -> mid silver-blue -> subtle bottom bounce)
let innerRimPath = makeSquirclePath(in: tileRect, inset: 3.0)
ctx.saveGState()
ctx.addPath(innerRimPath)
ctx.setLineWidth(5.5)
ctx.replacePathWithStrokedPath()
ctx.clip()

let rimColors = [
    CGColor(srgbRed: 1.0, green: 1.0, blue: 1.0, alpha: 0.52),
    CGColor(srgbRed: 0.68, green: 0.80, blue: 0.96, alpha: 0.22),
    CGColor(srgbRed: 0.85, green: 0.64, blue: 0.50, alpha: 0.16),
    CGColor(srgbRed: 0.75, green: 0.84, blue: 0.96, alpha: 0.30),
] as CFArray
if let rimGrad = CGGradient(colorsSpace: colorSpace, colors: rimColors, locations: [0.0, 0.35, 0.68, 1.0]) {
    ctx.drawLinearGradient(
        rimGrad,
        start: CGPoint(x: tileRect.minX + 40, y: tileRect.maxY),
        end: CGPoint(x: tileRect.maxX - 40, y: tileRect.minY),
        options: []
    )
}
ctx.restoreGState()

// Crisp 1.5px inner glass edge highlight
let fineRimPath = makeSquirclePath(in: tileRect, inset: 1.2)
ctx.addPath(fineRimPath)
ctx.setLineWidth(1.6)
ctx.setStrokeColor(CGColor(srgbRed: 1.0, green: 1.0, blue: 1.0, alpha: 0.24))
ctx.strokePath()

ctx.restoreGState()

// 5. Hairline dark outer stroke so the Liquid Glass tile stays crisp on Light Mode wallpaper
ctx.addPath(squircle)
ctx.setLineWidth(1.5)
ctx.setStrokeColor(CGColor(srgbRed: 0.03, green: 0.04, blue: 0.06, alpha: 0.35))
ctx.strokePath()

guard let outImage = ctx.makeImage() else {
    fputs("Failed to finalize CGImage\n", stderr)
    exit(1)
}

let rep = NSBitmapImageRep(cgImage: outImage)
guard let pngData = rep.representation(using: .png, properties: [:]) else {
    fputs("Failed to encode PNG\n", stderr)
    exit(1)
}

do {
    try pngData.write(to: URL(fileURLWithPath: outputPath))
    print("Wrote Liquid Glass icon: \(outputPath) (\(pngData.count) bytes)")
} catch {
    fputs("Failed to write PNG: \(error)\n", stderr)
    exit(1)
}
