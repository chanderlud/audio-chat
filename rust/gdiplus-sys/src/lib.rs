#![cfg(windows)]
#![allow(non_upper_case_globals, non_camel_case_types, non_snake_case)]
#![cfg_attr(not(test), no_std)]

use core::mem::MaybeUninit;

pub use winapi::ctypes::{c_int, c_void};
pub use winapi::shared::basetsd::{INT16, UINT16, UINT32, UINT_PTR, ULONG_PTR};
pub use winapi::shared::guiddef::{CLSID, GUID};
pub use winapi::shared::minwindef::{
    BOOL, BYTE, DWORD, HINSTANCE, HMETAFILE, HRGN, LPBYTE, UINT, WORD,
};
pub use winapi::shared::ntdef::{CHAR, HANDLE, INT, LANGID, LPWSTR, ULONG, WCHAR};
pub use winapi::shared::windef::{
    HBITMAP, HDC, HENHMETAFILE, HFONT, HICON, HPALETTE, HWND, RECTL, SIZEL,
};
pub use winapi::shared::wtypes::PROPID;
pub use winapi::um::commctrl::IStream;
pub use winapi::um::wingdi::{BITMAPINFO, LOGFONTA, LOGFONTW, METAHEADER};
pub use winapi::vc::vcruntime::size_t;

// #[cfg(test)]
// mod bindgen_tests;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct IDirectDrawSurface7 {
    _unused: [u8; 0],
}
extern "C" {
    #[link_name = "\u{1}GdipAlloc"]
    pub fn GdipAlloc(size: size_t) -> *mut c_void;
}
extern "C" {
    #[link_name = "\u{1}GdipFree"]
    pub fn GdipFree(ptr: *mut c_void);
}
extern "C" {
    #[link_name = "\u{1}GdipCreatePath"]
    pub fn GdipCreatePath(brushMode: GpFillMode, path: *mut *mut GpPath) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipCreatePath2"]
    pub fn GdipCreatePath2(
        arg1: *const GpPointF,
        arg2: *const BYTE,
        arg3: INT,
        arg4: GpFillMode,
        path: *mut *mut GpPath,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipCreatePath2I"]
    pub fn GdipCreatePath2I(
        arg1: *const GpPoint,
        arg2: *const BYTE,
        arg3: INT,
        arg4: GpFillMode,
        path: *mut *mut GpPath,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipClonePath"]
    pub fn GdipClonePath(path: *mut GpPath, clonePath: *mut *mut GpPath) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipDeletePath"]
    pub fn GdipDeletePath(path: *mut GpPath) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipResetPath"]
    pub fn GdipResetPath(path: *mut GpPath) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetPointCount"]
    pub fn GdipGetPointCount(path: *mut GpPath, count: *mut INT) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetPathTypes"]
    pub fn GdipGetPathTypes(path: *mut GpPath, types: *mut BYTE, count: INT) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetPathPoints"]
    pub fn GdipGetPathPoints(arg1: *mut GpPath, points: *mut GpPointF, count: INT) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetPathPointsI"]
    pub fn GdipGetPathPointsI(arg1: *mut GpPath, points: *mut GpPoint, count: INT) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetPathFillMode"]
    pub fn GdipGetPathFillMode(path: *mut GpPath, fillmode: *mut GpFillMode) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipSetPathFillMode"]
    pub fn GdipSetPathFillMode(path: *mut GpPath, fillmode: GpFillMode) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetPathData"]
    pub fn GdipGetPathData(path: *mut GpPath, pathData: *mut GpPathData) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipStartPathFigure"]
    pub fn GdipStartPathFigure(path: *mut GpPath) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipClosePathFigure"]
    pub fn GdipClosePathFigure(path: *mut GpPath) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipClosePathFigures"]
    pub fn GdipClosePathFigures(path: *mut GpPath) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipSetPathMarker"]
    pub fn GdipSetPathMarker(path: *mut GpPath) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipClearPathMarkers"]
    pub fn GdipClearPathMarkers(path: *mut GpPath) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipReversePath"]
    pub fn GdipReversePath(path: *mut GpPath) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetPathLastPoint"]
    pub fn GdipGetPathLastPoint(path: *mut GpPath, lastPoint: *mut GpPointF) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipAddPathLine"]
    pub fn GdipAddPathLine(path: *mut GpPath, x1: REAL, y1: REAL, x2: REAL, y2: REAL) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipAddPathLine2"]
    pub fn GdipAddPathLine2(path: *mut GpPath, points: *const GpPointF, count: INT) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipAddPathArc"]
    pub fn GdipAddPathArc(
        path: *mut GpPath,
        x: REAL,
        y: REAL,
        width: REAL,
        height: REAL,
        startAngle: REAL,
        sweepAngle: REAL,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipAddPathBezier"]
    pub fn GdipAddPathBezier(
        path: *mut GpPath,
        x1: REAL,
        y1: REAL,
        x2: REAL,
        y2: REAL,
        x3: REAL,
        y3: REAL,
        x4: REAL,
        y4: REAL,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipAddPathBeziers"]
    pub fn GdipAddPathBeziers(path: *mut GpPath, points: *const GpPointF, count: INT) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipAddPathCurve"]
    pub fn GdipAddPathCurve(path: *mut GpPath, points: *const GpPointF, count: INT) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipAddPathCurve2"]
    pub fn GdipAddPathCurve2(
        path: *mut GpPath,
        points: *const GpPointF,
        count: INT,
        tension: REAL,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipAddPathCurve3"]
    pub fn GdipAddPathCurve3(
        path: *mut GpPath,
        points: *const GpPointF,
        count: INT,
        offset: INT,
        numberOfSegments: INT,
        tension: REAL,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipAddPathClosedCurve"]
    pub fn GdipAddPathClosedCurve(
        path: *mut GpPath,
        points: *const GpPointF,
        count: INT,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipAddPathClosedCurve2"]
    pub fn GdipAddPathClosedCurve2(
        path: *mut GpPath,
        points: *const GpPointF,
        count: INT,
        tension: REAL,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipAddPathRectangle"]
    pub fn GdipAddPathRectangle(
        path: *mut GpPath,
        x: REAL,
        y: REAL,
        width: REAL,
        height: REAL,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipAddPathRectangles"]
    pub fn GdipAddPathRectangles(path: *mut GpPath, rects: *const GpRectF, count: INT) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipAddPathEllipse"]
    pub fn GdipAddPathEllipse(
        path: *mut GpPath,
        x: REAL,
        y: REAL,
        width: REAL,
        height: REAL,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipAddPathPie"]
    pub fn GdipAddPathPie(
        path: *mut GpPath,
        x: REAL,
        y: REAL,
        width: REAL,
        height: REAL,
        startAngle: REAL,
        sweepAngle: REAL,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipAddPathPolygon"]
    pub fn GdipAddPathPolygon(path: *mut GpPath, points: *const GpPointF, count: INT) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipAddPathPath"]
    pub fn GdipAddPathPath(path: *mut GpPath, addingPath: *const GpPath, connect: BOOL)
        -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipAddPathString"]
    pub fn GdipAddPathString(
        path: *mut GpPath,
        string: *const WCHAR,
        length: INT,
        family: *const GpFontFamily,
        style: INT,
        emSize: REAL,
        layoutRect: *const RectF,
        format: *const GpStringFormat,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipAddPathStringI"]
    pub fn GdipAddPathStringI(
        path: *mut GpPath,
        string: *const WCHAR,
        length: INT,
        family: *const GpFontFamily,
        style: INT,
        emSize: REAL,
        layoutRect: *const Rect,
        format: *const GpStringFormat,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipAddPathLineI"]
    pub fn GdipAddPathLineI(path: *mut GpPath, x1: INT, y1: INT, x2: INT, y2: INT) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipAddPathLine2I"]
    pub fn GdipAddPathLine2I(path: *mut GpPath, points: *const GpPoint, count: INT) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipAddPathArcI"]
    pub fn GdipAddPathArcI(
        path: *mut GpPath,
        x: INT,
        y: INT,
        width: INT,
        height: INT,
        startAngle: REAL,
        sweepAngle: REAL,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipAddPathBezierI"]
    pub fn GdipAddPathBezierI(
        path: *mut GpPath,
        x1: INT,
        y1: INT,
        x2: INT,
        y2: INT,
        x3: INT,
        y3: INT,
        x4: INT,
        y4: INT,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipAddPathBeziersI"]
    pub fn GdipAddPathBeziersI(path: *mut GpPath, points: *const GpPoint, count: INT) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipAddPathCurveI"]
    pub fn GdipAddPathCurveI(path: *mut GpPath, points: *const GpPoint, count: INT) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipAddPathCurve2I"]
    pub fn GdipAddPathCurve2I(
        path: *mut GpPath,
        points: *const GpPoint,
        count: INT,
        tension: REAL,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipAddPathCurve3I"]
    pub fn GdipAddPathCurve3I(
        path: *mut GpPath,
        points: *const GpPoint,
        count: INT,
        offset: INT,
        numberOfSegments: INT,
        tension: REAL,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipAddPathClosedCurveI"]
    pub fn GdipAddPathClosedCurveI(
        path: *mut GpPath,
        points: *const GpPoint,
        count: INT,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipAddPathClosedCurve2I"]
    pub fn GdipAddPathClosedCurve2I(
        path: *mut GpPath,
        points: *const GpPoint,
        count: INT,
        tension: REAL,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipAddPathRectangleI"]
    pub fn GdipAddPathRectangleI(
        path: *mut GpPath,
        x: INT,
        y: INT,
        width: INT,
        height: INT,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipAddPathRectanglesI"]
    pub fn GdipAddPathRectanglesI(path: *mut GpPath, rects: *const GpRect, count: INT) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipAddPathEllipseI"]
    pub fn GdipAddPathEllipseI(
        path: *mut GpPath,
        x: INT,
        y: INT,
        width: INT,
        height: INT,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipAddPathPieI"]
    pub fn GdipAddPathPieI(
        path: *mut GpPath,
        x: INT,
        y: INT,
        width: INT,
        height: INT,
        startAngle: REAL,
        sweepAngle: REAL,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipAddPathPolygonI"]
    pub fn GdipAddPathPolygonI(path: *mut GpPath, points: *const GpPoint, count: INT) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipFlattenPath"]
    pub fn GdipFlattenPath(path: *mut GpPath, matrix: *mut GpMatrix, flatness: REAL) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipWindingModeOutline"]
    pub fn GdipWindingModeOutline(
        path: *mut GpPath,
        matrix: *mut GpMatrix,
        flatness: REAL,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipWidenPath"]
    pub fn GdipWidenPath(
        nativePath: *mut GpPath,
        pen: *mut GpPen,
        matrix: *mut GpMatrix,
        flatness: REAL,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipWarpPath"]
    pub fn GdipWarpPath(
        path: *mut GpPath,
        matrix: *mut GpMatrix,
        points: *const GpPointF,
        count: INT,
        srcx: REAL,
        srcy: REAL,
        srcwidth: REAL,
        srcheight: REAL,
        warpMode: WarpMode,
        flatness: REAL,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipTransformPath"]
    pub fn GdipTransformPath(path: *mut GpPath, matrix: *mut GpMatrix) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetPathWorldBounds"]
    pub fn GdipGetPathWorldBounds(
        path: *mut GpPath,
        bounds: *mut GpRectF,
        matrix: *const GpMatrix,
        pen: *const GpPen,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetPathWorldBoundsI"]
    pub fn GdipGetPathWorldBoundsI(
        path: *mut GpPath,
        bounds: *mut GpRect,
        matrix: *const GpMatrix,
        pen: *const GpPen,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipIsVisiblePathPoint"]
    pub fn GdipIsVisiblePathPoint(
        path: *mut GpPath,
        x: REAL,
        y: REAL,
        graphics: *mut GpGraphics,
        result: *mut BOOL,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipIsVisiblePathPointI"]
    pub fn GdipIsVisiblePathPointI(
        path: *mut GpPath,
        x: INT,
        y: INT,
        graphics: *mut GpGraphics,
        result: *mut BOOL,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipIsOutlineVisiblePathPoint"]
    pub fn GdipIsOutlineVisiblePathPoint(
        path: *mut GpPath,
        x: REAL,
        y: REAL,
        pen: *mut GpPen,
        graphics: *mut GpGraphics,
        result: *mut BOOL,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipIsOutlineVisiblePathPointI"]
    pub fn GdipIsOutlineVisiblePathPointI(
        path: *mut GpPath,
        x: INT,
        y: INT,
        pen: *mut GpPen,
        graphics: *mut GpGraphics,
        result: *mut BOOL,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipCreatePathIter"]
    pub fn GdipCreatePathIter(iterator: *mut *mut GpPathIterator, path: *mut GpPath) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipDeletePathIter"]
    pub fn GdipDeletePathIter(iterator: *mut GpPathIterator) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipPathIterNextSubpath"]
    pub fn GdipPathIterNextSubpath(
        iterator: *mut GpPathIterator,
        resultCount: *mut INT,
        startIndex: *mut INT,
        endIndex: *mut INT,
        isClosed: *mut BOOL,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipPathIterNextSubpathPath"]
    pub fn GdipPathIterNextSubpathPath(
        iterator: *mut GpPathIterator,
        resultCount: *mut INT,
        path: *mut GpPath,
        isClosed: *mut BOOL,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipPathIterNextPathType"]
    pub fn GdipPathIterNextPathType(
        iterator: *mut GpPathIterator,
        resultCount: *mut INT,
        pathType: *mut BYTE,
        startIndex: *mut INT,
        endIndex: *mut INT,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipPathIterNextMarker"]
    pub fn GdipPathIterNextMarker(
        iterator: *mut GpPathIterator,
        resultCount: *mut INT,
        startIndex: *mut INT,
        endIndex: *mut INT,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipPathIterNextMarkerPath"]
    pub fn GdipPathIterNextMarkerPath(
        iterator: *mut GpPathIterator,
        resultCount: *mut INT,
        path: *mut GpPath,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipPathIterGetCount"]
    pub fn GdipPathIterGetCount(iterator: *mut GpPathIterator, count: *mut INT) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipPathIterGetSubpathCount"]
    pub fn GdipPathIterGetSubpathCount(iterator: *mut GpPathIterator, count: *mut INT) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipPathIterIsValid"]
    pub fn GdipPathIterIsValid(iterator: *mut GpPathIterator, valid: *mut BOOL) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipPathIterHasCurve"]
    pub fn GdipPathIterHasCurve(iterator: *mut GpPathIterator, hasCurve: *mut BOOL) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipPathIterRewind"]
    pub fn GdipPathIterRewind(iterator: *mut GpPathIterator) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipPathIterEnumerate"]
    pub fn GdipPathIterEnumerate(
        iterator: *mut GpPathIterator,
        resultCount: *mut INT,
        points: *mut GpPointF,
        types: *mut BYTE,
        count: INT,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipPathIterCopyData"]
    pub fn GdipPathIterCopyData(
        iterator: *mut GpPathIterator,
        resultCount: *mut INT,
        points: *mut GpPointF,
        types: *mut BYTE,
        startIndex: INT,
        endIndex: INT,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipCreateMatrix"]
    pub fn GdipCreateMatrix(matrix: *mut *mut GpMatrix) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipCreateMatrix2"]
    pub fn GdipCreateMatrix2(
        m11: REAL,
        m12: REAL,
        m21: REAL,
        m22: REAL,
        dx: REAL,
        dy: REAL,
        matrix: *mut *mut GpMatrix,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipCreateMatrix3"]
    pub fn GdipCreateMatrix3(
        rect: *const GpRectF,
        dstplg: *const GpPointF,
        matrix: *mut *mut GpMatrix,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipCreateMatrix3I"]
    pub fn GdipCreateMatrix3I(
        rect: *const GpRect,
        dstplg: *const GpPoint,
        matrix: *mut *mut GpMatrix,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipCloneMatrix"]
    pub fn GdipCloneMatrix(matrix: *mut GpMatrix, cloneMatrix: *mut *mut GpMatrix) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipDeleteMatrix"]
    pub fn GdipDeleteMatrix(matrix: *mut GpMatrix) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipSetMatrixElements"]
    pub fn GdipSetMatrixElements(
        matrix: *mut GpMatrix,
        m11: REAL,
        m12: REAL,
        m21: REAL,
        m22: REAL,
        dx: REAL,
        dy: REAL,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipMultiplyMatrix"]
    pub fn GdipMultiplyMatrix(
        matrix: *mut GpMatrix,
        matrix2: *mut GpMatrix,
        order: GpMatrixOrder,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipTranslateMatrix"]
    pub fn GdipTranslateMatrix(
        matrix: *mut GpMatrix,
        offsetX: REAL,
        offsetY: REAL,
        order: GpMatrixOrder,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipScaleMatrix"]
    pub fn GdipScaleMatrix(
        matrix: *mut GpMatrix,
        scaleX: REAL,
        scaleY: REAL,
        order: GpMatrixOrder,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipRotateMatrix"]
    pub fn GdipRotateMatrix(matrix: *mut GpMatrix, angle: REAL, order: GpMatrixOrder) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipShearMatrix"]
    pub fn GdipShearMatrix(
        matrix: *mut GpMatrix,
        shearX: REAL,
        shearY: REAL,
        order: GpMatrixOrder,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipInvertMatrix"]
    pub fn GdipInvertMatrix(matrix: *mut GpMatrix) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipTransformMatrixPoints"]
    pub fn GdipTransformMatrixPoints(
        matrix: *mut GpMatrix,
        pts: *mut GpPointF,
        count: INT,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipTransformMatrixPointsI"]
    pub fn GdipTransformMatrixPointsI(
        matrix: *mut GpMatrix,
        pts: *mut GpPoint,
        count: INT,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipVectorTransformMatrixPoints"]
    pub fn GdipVectorTransformMatrixPoints(
        matrix: *mut GpMatrix,
        pts: *mut GpPointF,
        count: INT,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipVectorTransformMatrixPointsI"]
    pub fn GdipVectorTransformMatrixPointsI(
        matrix: *mut GpMatrix,
        pts: *mut GpPoint,
        count: INT,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetMatrixElements"]
    pub fn GdipGetMatrixElements(matrix: *const GpMatrix, matrixOut: *mut REAL) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipIsMatrixInvertible"]
    pub fn GdipIsMatrixInvertible(matrix: *const GpMatrix, result: *mut BOOL) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipIsMatrixIdentity"]
    pub fn GdipIsMatrixIdentity(matrix: *const GpMatrix, result: *mut BOOL) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipIsMatrixEqual"]
    pub fn GdipIsMatrixEqual(
        matrix: *const GpMatrix,
        matrix2: *const GpMatrix,
        result: *mut BOOL,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipCreateRegion"]
    pub fn GdipCreateRegion(region: *mut *mut GpRegion) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipCreateRegionRect"]
    pub fn GdipCreateRegionRect(rect: *const GpRectF, region: *mut *mut GpRegion) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipCreateRegionRectI"]
    pub fn GdipCreateRegionRectI(rect: *const GpRect, region: *mut *mut GpRegion) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipCreateRegionPath"]
    pub fn GdipCreateRegionPath(path: *mut GpPath, region: *mut *mut GpRegion) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipCreateRegionRgnData"]
    pub fn GdipCreateRegionRgnData(
        regionData: *const BYTE,
        size: INT,
        region: *mut *mut GpRegion,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipCreateRegionHrgn"]
    pub fn GdipCreateRegionHrgn(hRgn: HRGN, region: *mut *mut GpRegion) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipCloneRegion"]
    pub fn GdipCloneRegion(region: *mut GpRegion, cloneRegion: *mut *mut GpRegion) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipDeleteRegion"]
    pub fn GdipDeleteRegion(region: *mut GpRegion) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipSetInfinite"]
    pub fn GdipSetInfinite(region: *mut GpRegion) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipSetEmpty"]
    pub fn GdipSetEmpty(region: *mut GpRegion) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipCombineRegionRect"]
    pub fn GdipCombineRegionRect(
        region: *mut GpRegion,
        rect: *const GpRectF,
        combineMode: CombineMode,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipCombineRegionRectI"]
    pub fn GdipCombineRegionRectI(
        region: *mut GpRegion,
        rect: *const GpRect,
        combineMode: CombineMode,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipCombineRegionPath"]
    pub fn GdipCombineRegionPath(
        region: *mut GpRegion,
        path: *mut GpPath,
        combineMode: CombineMode,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipCombineRegionRegion"]
    pub fn GdipCombineRegionRegion(
        region: *mut GpRegion,
        region2: *mut GpRegion,
        combineMode: CombineMode,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipTranslateRegion"]
    pub fn GdipTranslateRegion(region: *mut GpRegion, dx: REAL, dy: REAL) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipTranslateRegionI"]
    pub fn GdipTranslateRegionI(region: *mut GpRegion, dx: INT, dy: INT) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipTransformRegion"]
    pub fn GdipTransformRegion(region: *mut GpRegion, matrix: *mut GpMatrix) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetRegionBounds"]
    pub fn GdipGetRegionBounds(
        region: *mut GpRegion,
        graphics: *mut GpGraphics,
        rect: *mut GpRectF,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetRegionBoundsI"]
    pub fn GdipGetRegionBoundsI(
        region: *mut GpRegion,
        graphics: *mut GpGraphics,
        rect: *mut GpRect,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetRegionHRgn"]
    pub fn GdipGetRegionHRgn(
        region: *mut GpRegion,
        graphics: *mut GpGraphics,
        hRgn: *mut HRGN,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipIsEmptyRegion"]
    pub fn GdipIsEmptyRegion(
        region: *mut GpRegion,
        graphics: *mut GpGraphics,
        result: *mut BOOL,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipIsInfiniteRegion"]
    pub fn GdipIsInfiniteRegion(
        region: *mut GpRegion,
        graphics: *mut GpGraphics,
        result: *mut BOOL,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipIsEqualRegion"]
    pub fn GdipIsEqualRegion(
        region: *mut GpRegion,
        region2: *mut GpRegion,
        graphics: *mut GpGraphics,
        result: *mut BOOL,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetRegionDataSize"]
    pub fn GdipGetRegionDataSize(region: *mut GpRegion, bufferSize: *mut UINT) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetRegionData"]
    pub fn GdipGetRegionData(
        region: *mut GpRegion,
        buffer: *mut BYTE,
        bufferSize: UINT,
        sizeFilled: *mut UINT,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipIsVisibleRegionPoint"]
    pub fn GdipIsVisibleRegionPoint(
        region: *mut GpRegion,
        x: REAL,
        y: REAL,
        graphics: *mut GpGraphics,
        result: *mut BOOL,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipIsVisibleRegionPointI"]
    pub fn GdipIsVisibleRegionPointI(
        region: *mut GpRegion,
        x: INT,
        y: INT,
        graphics: *mut GpGraphics,
        result: *mut BOOL,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipIsVisibleRegionRect"]
    pub fn GdipIsVisibleRegionRect(
        region: *mut GpRegion,
        x: REAL,
        y: REAL,
        width: REAL,
        height: REAL,
        graphics: *mut GpGraphics,
        result: *mut BOOL,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipIsVisibleRegionRectI"]
    pub fn GdipIsVisibleRegionRectI(
        region: *mut GpRegion,
        x: INT,
        y: INT,
        width: INT,
        height: INT,
        graphics: *mut GpGraphics,
        result: *mut BOOL,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetRegionScansCount"]
    pub fn GdipGetRegionScansCount(
        region: *mut GpRegion,
        count: *mut UINT,
        matrix: *mut GpMatrix,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetRegionScans"]
    pub fn GdipGetRegionScans(
        region: *mut GpRegion,
        rects: *mut GpRectF,
        count: *mut INT,
        matrix: *mut GpMatrix,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetRegionScansI"]
    pub fn GdipGetRegionScansI(
        region: *mut GpRegion,
        rects: *mut GpRect,
        count: *mut INT,
        matrix: *mut GpMatrix,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipCloneBrush"]
    pub fn GdipCloneBrush(brush: *mut GpBrush, cloneBrush: *mut *mut GpBrush) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipDeleteBrush"]
    pub fn GdipDeleteBrush(brush: *mut GpBrush) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetBrushType"]
    pub fn GdipGetBrushType(brush: *mut GpBrush, type_: *mut GpBrushType) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipCreateHatchBrush"]
    pub fn GdipCreateHatchBrush(
        hatchstyle: GpHatchStyle,
        forecol: ARGB,
        backcol: ARGB,
        brush: *mut *mut GpHatch,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetHatchStyle"]
    pub fn GdipGetHatchStyle(brush: *mut GpHatch, hatchstyle: *mut GpHatchStyle) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetHatchForegroundColor"]
    pub fn GdipGetHatchForegroundColor(brush: *mut GpHatch, forecol: *mut ARGB) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetHatchBackgroundColor"]
    pub fn GdipGetHatchBackgroundColor(brush: *mut GpHatch, backcol: *mut ARGB) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipCreateTexture"]
    pub fn GdipCreateTexture(
        image: *mut GpImage,
        wrapmode: GpWrapMode,
        texture: *mut *mut GpTexture,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipCreateTexture2"]
    pub fn GdipCreateTexture2(
        image: *mut GpImage,
        wrapmode: GpWrapMode,
        x: REAL,
        y: REAL,
        width: REAL,
        height: REAL,
        texture: *mut *mut GpTexture,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipCreateTextureIA"]
    pub fn GdipCreateTextureIA(
        image: *mut GpImage,
        imageAttributes: *const GpImageAttributes,
        x: REAL,
        y: REAL,
        width: REAL,
        height: REAL,
        texture: *mut *mut GpTexture,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipCreateTexture2I"]
    pub fn GdipCreateTexture2I(
        image: *mut GpImage,
        wrapmode: GpWrapMode,
        x: INT,
        y: INT,
        width: INT,
        height: INT,
        texture: *mut *mut GpTexture,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipCreateTextureIAI"]
    pub fn GdipCreateTextureIAI(
        image: *mut GpImage,
        imageAttributes: *const GpImageAttributes,
        x: INT,
        y: INT,
        width: INT,
        height: INT,
        texture: *mut *mut GpTexture,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetTextureTransform"]
    pub fn GdipGetTextureTransform(brush: *mut GpTexture, matrix: *mut GpMatrix) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipSetTextureTransform"]
    pub fn GdipSetTextureTransform(brush: *mut GpTexture, matrix: *const GpMatrix) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipResetTextureTransform"]
    pub fn GdipResetTextureTransform(brush: *mut GpTexture) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipMultiplyTextureTransform"]
    pub fn GdipMultiplyTextureTransform(
        brush: *mut GpTexture,
        matrix: *const GpMatrix,
        order: GpMatrixOrder,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipTranslateTextureTransform"]
    pub fn GdipTranslateTextureTransform(
        brush: *mut GpTexture,
        dx: REAL,
        dy: REAL,
        order: GpMatrixOrder,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipScaleTextureTransform"]
    pub fn GdipScaleTextureTransform(
        brush: *mut GpTexture,
        sx: REAL,
        sy: REAL,
        order: GpMatrixOrder,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipRotateTextureTransform"]
    pub fn GdipRotateTextureTransform(
        brush: *mut GpTexture,
        angle: REAL,
        order: GpMatrixOrder,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipSetTextureWrapMode"]
    pub fn GdipSetTextureWrapMode(brush: *mut GpTexture, wrapmode: GpWrapMode) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetTextureWrapMode"]
    pub fn GdipGetTextureWrapMode(brush: *mut GpTexture, wrapmode: *mut GpWrapMode) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetTextureImage"]
    pub fn GdipGetTextureImage(brush: *mut GpTexture, image: *mut *mut GpImage) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipCreateSolidFill"]
    pub fn GdipCreateSolidFill(color: ARGB, brush: *mut *mut GpSolidFill) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipSetSolidFillColor"]
    pub fn GdipSetSolidFillColor(brush: *mut GpSolidFill, color: ARGB) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetSolidFillColor"]
    pub fn GdipGetSolidFillColor(brush: *mut GpSolidFill, color: *mut ARGB) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipCreateLineBrush"]
    pub fn GdipCreateLineBrush(
        point1: *const GpPointF,
        point2: *const GpPointF,
        color1: ARGB,
        color2: ARGB,
        wrapMode: GpWrapMode,
        lineGradient: *mut *mut GpLineGradient,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipCreateLineBrushI"]
    pub fn GdipCreateLineBrushI(
        point1: *const GpPoint,
        point2: *const GpPoint,
        color1: ARGB,
        color2: ARGB,
        wrapMode: GpWrapMode,
        lineGradient: *mut *mut GpLineGradient,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipCreateLineBrushFromRect"]
    pub fn GdipCreateLineBrushFromRect(
        rect: *const GpRectF,
        color1: ARGB,
        color2: ARGB,
        mode: LinearGradientMode,
        wrapMode: GpWrapMode,
        lineGradient: *mut *mut GpLineGradient,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipCreateLineBrushFromRectI"]
    pub fn GdipCreateLineBrushFromRectI(
        rect: *const GpRect,
        color1: ARGB,
        color2: ARGB,
        mode: LinearGradientMode,
        wrapMode: GpWrapMode,
        lineGradient: *mut *mut GpLineGradient,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipCreateLineBrushFromRectWithAngle"]
    pub fn GdipCreateLineBrushFromRectWithAngle(
        rect: *const GpRectF,
        color1: ARGB,
        color2: ARGB,
        angle: REAL,
        isAngleScalable: BOOL,
        wrapMode: GpWrapMode,
        lineGradient: *mut *mut GpLineGradient,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipCreateLineBrushFromRectWithAngleI"]
    pub fn GdipCreateLineBrushFromRectWithAngleI(
        rect: *const GpRect,
        color1: ARGB,
        color2: ARGB,
        angle: REAL,
        isAngleScalable: BOOL,
        wrapMode: GpWrapMode,
        lineGradient: *mut *mut GpLineGradient,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipSetLineColors"]
    pub fn GdipSetLineColors(brush: *mut GpLineGradient, color1: ARGB, color2: ARGB) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetLineColors"]
    pub fn GdipGetLineColors(brush: *mut GpLineGradient, colors: *mut ARGB) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetLineRect"]
    pub fn GdipGetLineRect(brush: *mut GpLineGradient, rect: *mut GpRectF) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetLineRectI"]
    pub fn GdipGetLineRectI(brush: *mut GpLineGradient, rect: *mut GpRect) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipSetLineGammaCorrection"]
    pub fn GdipSetLineGammaCorrection(
        brush: *mut GpLineGradient,
        useGammaCorrection: BOOL,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetLineGammaCorrection"]
    pub fn GdipGetLineGammaCorrection(
        brush: *mut GpLineGradient,
        useGammaCorrection: *mut BOOL,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetLineBlendCount"]
    pub fn GdipGetLineBlendCount(brush: *mut GpLineGradient, count: *mut INT) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetLineBlend"]
    pub fn GdipGetLineBlend(
        brush: *mut GpLineGradient,
        blend: *mut REAL,
        positions: *mut REAL,
        count: INT,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipSetLineBlend"]
    pub fn GdipSetLineBlend(
        brush: *mut GpLineGradient,
        blend: *const REAL,
        positions: *const REAL,
        count: INT,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetLinePresetBlendCount"]
    pub fn GdipGetLinePresetBlendCount(brush: *mut GpLineGradient, count: *mut INT) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetLinePresetBlend"]
    pub fn GdipGetLinePresetBlend(
        brush: *mut GpLineGradient,
        blend: *mut ARGB,
        positions: *mut REAL,
        count: INT,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipSetLinePresetBlend"]
    pub fn GdipSetLinePresetBlend(
        brush: *mut GpLineGradient,
        blend: *const ARGB,
        positions: *const REAL,
        count: INT,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipSetLineSigmaBlend"]
    pub fn GdipSetLineSigmaBlend(brush: *mut GpLineGradient, focus: REAL, scale: REAL) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipSetLineLinearBlend"]
    pub fn GdipSetLineLinearBlend(brush: *mut GpLineGradient, focus: REAL, scale: REAL)
        -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipSetLineWrapMode"]
    pub fn GdipSetLineWrapMode(brush: *mut GpLineGradient, wrapmode: GpWrapMode) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetLineWrapMode"]
    pub fn GdipGetLineWrapMode(brush: *mut GpLineGradient, wrapmode: *mut GpWrapMode) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetLineTransform"]
    pub fn GdipGetLineTransform(brush: *mut GpLineGradient, matrix: *mut GpMatrix) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipSetLineTransform"]
    pub fn GdipSetLineTransform(brush: *mut GpLineGradient, matrix: *const GpMatrix) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipResetLineTransform"]
    pub fn GdipResetLineTransform(brush: *mut GpLineGradient) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipMultiplyLineTransform"]
    pub fn GdipMultiplyLineTransform(
        brush: *mut GpLineGradient,
        matrix: *const GpMatrix,
        order: GpMatrixOrder,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipTranslateLineTransform"]
    pub fn GdipTranslateLineTransform(
        brush: *mut GpLineGradient,
        dx: REAL,
        dy: REAL,
        order: GpMatrixOrder,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipScaleLineTransform"]
    pub fn GdipScaleLineTransform(
        brush: *mut GpLineGradient,
        sx: REAL,
        sy: REAL,
        order: GpMatrixOrder,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipRotateLineTransform"]
    pub fn GdipRotateLineTransform(
        brush: *mut GpLineGradient,
        angle: REAL,
        order: GpMatrixOrder,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipCreatePathGradient"]
    pub fn GdipCreatePathGradient(
        points: *const GpPointF,
        count: INT,
        wrapMode: GpWrapMode,
        polyGradient: *mut *mut GpPathGradient,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipCreatePathGradientI"]
    pub fn GdipCreatePathGradientI(
        points: *const GpPoint,
        count: INT,
        wrapMode: GpWrapMode,
        polyGradient: *mut *mut GpPathGradient,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipCreatePathGradientFromPath"]
    pub fn GdipCreatePathGradientFromPath(
        path: *const GpPath,
        polyGradient: *mut *mut GpPathGradient,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetPathGradientCenterColor"]
    pub fn GdipGetPathGradientCenterColor(
        brush: *mut GpPathGradient,
        colors: *mut ARGB,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipSetPathGradientCenterColor"]
    pub fn GdipSetPathGradientCenterColor(brush: *mut GpPathGradient, colors: ARGB) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetPathGradientSurroundColorsWithCount"]
    pub fn GdipGetPathGradientSurroundColorsWithCount(
        brush: *mut GpPathGradient,
        color: *mut ARGB,
        count: *mut INT,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipSetPathGradientSurroundColorsWithCount"]
    pub fn GdipSetPathGradientSurroundColorsWithCount(
        brush: *mut GpPathGradient,
        color: *const ARGB,
        count: *mut INT,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetPathGradientPath"]
    pub fn GdipGetPathGradientPath(brush: *mut GpPathGradient, path: *mut GpPath) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipSetPathGradientPath"]
    pub fn GdipSetPathGradientPath(brush: *mut GpPathGradient, path: *const GpPath) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetPathGradientCenterPoint"]
    pub fn GdipGetPathGradientCenterPoint(
        brush: *mut GpPathGradient,
        points: *mut GpPointF,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetPathGradientCenterPointI"]
    pub fn GdipGetPathGradientCenterPointI(
        brush: *mut GpPathGradient,
        points: *mut GpPoint,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipSetPathGradientCenterPoint"]
    pub fn GdipSetPathGradientCenterPoint(
        brush: *mut GpPathGradient,
        points: *const GpPointF,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipSetPathGradientCenterPointI"]
    pub fn GdipSetPathGradientCenterPointI(
        brush: *mut GpPathGradient,
        points: *const GpPoint,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetPathGradientRect"]
    pub fn GdipGetPathGradientRect(brush: *mut GpPathGradient, rect: *mut GpRectF) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetPathGradientRectI"]
    pub fn GdipGetPathGradientRectI(brush: *mut GpPathGradient, rect: *mut GpRect) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetPathGradientPointCount"]
    pub fn GdipGetPathGradientPointCount(brush: *mut GpPathGradient, count: *mut INT) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetPathGradientSurroundColorCount"]
    pub fn GdipGetPathGradientSurroundColorCount(
        brush: *mut GpPathGradient,
        count: *mut INT,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipSetPathGradientGammaCorrection"]
    pub fn GdipSetPathGradientGammaCorrection(
        brush: *mut GpPathGradient,
        useGammaCorrection: BOOL,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetPathGradientGammaCorrection"]
    pub fn GdipGetPathGradientGammaCorrection(
        brush: *mut GpPathGradient,
        useGammaCorrection: *mut BOOL,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetPathGradientBlendCount"]
    pub fn GdipGetPathGradientBlendCount(brush: *mut GpPathGradient, count: *mut INT) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetPathGradientBlend"]
    pub fn GdipGetPathGradientBlend(
        brush: *mut GpPathGradient,
        blend: *mut REAL,
        positions: *mut REAL,
        count: INT,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipSetPathGradientBlend"]
    pub fn GdipSetPathGradientBlend(
        brush: *mut GpPathGradient,
        blend: *const REAL,
        positions: *const REAL,
        count: INT,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetPathGradientPresetBlendCount"]
    pub fn GdipGetPathGradientPresetBlendCount(
        brush: *mut GpPathGradient,
        count: *mut INT,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetPathGradientPresetBlend"]
    pub fn GdipGetPathGradientPresetBlend(
        brush: *mut GpPathGradient,
        blend: *mut ARGB,
        positions: *mut REAL,
        count: INT,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipSetPathGradientPresetBlend"]
    pub fn GdipSetPathGradientPresetBlend(
        brush: *mut GpPathGradient,
        blend: *const ARGB,
        positions: *const REAL,
        count: INT,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipSetPathGradientSigmaBlend"]
    pub fn GdipSetPathGradientSigmaBlend(
        brush: *mut GpPathGradient,
        focus: REAL,
        scale: REAL,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipSetPathGradientLinearBlend"]
    pub fn GdipSetPathGradientLinearBlend(
        brush: *mut GpPathGradient,
        focus: REAL,
        scale: REAL,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetPathGradientWrapMode"]
    pub fn GdipGetPathGradientWrapMode(
        brush: *mut GpPathGradient,
        wrapmode: *mut GpWrapMode,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipSetPathGradientWrapMode"]
    pub fn GdipSetPathGradientWrapMode(
        brush: *mut GpPathGradient,
        wrapmode: GpWrapMode,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetPathGradientTransform"]
    pub fn GdipGetPathGradientTransform(
        brush: *mut GpPathGradient,
        matrix: *mut GpMatrix,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipSetPathGradientTransform"]
    pub fn GdipSetPathGradientTransform(
        brush: *mut GpPathGradient,
        matrix: *mut GpMatrix,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipResetPathGradientTransform"]
    pub fn GdipResetPathGradientTransform(brush: *mut GpPathGradient) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipMultiplyPathGradientTransform"]
    pub fn GdipMultiplyPathGradientTransform(
        brush: *mut GpPathGradient,
        matrix: *const GpMatrix,
        order: GpMatrixOrder,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipTranslatePathGradientTransform"]
    pub fn GdipTranslatePathGradientTransform(
        brush: *mut GpPathGradient,
        dx: REAL,
        dy: REAL,
        order: GpMatrixOrder,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipScalePathGradientTransform"]
    pub fn GdipScalePathGradientTransform(
        brush: *mut GpPathGradient,
        sx: REAL,
        sy: REAL,
        order: GpMatrixOrder,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipRotatePathGradientTransform"]
    pub fn GdipRotatePathGradientTransform(
        brush: *mut GpPathGradient,
        angle: REAL,
        order: GpMatrixOrder,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetPathGradientFocusScales"]
    pub fn GdipGetPathGradientFocusScales(
        brush: *mut GpPathGradient,
        xScale: *mut REAL,
        yScale: *mut REAL,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipSetPathGradientFocusScales"]
    pub fn GdipSetPathGradientFocusScales(
        brush: *mut GpPathGradient,
        xScale: REAL,
        yScale: REAL,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipCreatePen1"]
    pub fn GdipCreatePen1(color: ARGB, width: REAL, unit: GpUnit, pen: *mut *mut GpPen)
        -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipCreatePen2"]
    pub fn GdipCreatePen2(
        brush: *mut GpBrush,
        width: REAL,
        unit: GpUnit,
        pen: *mut *mut GpPen,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipClonePen"]
    pub fn GdipClonePen(pen: *mut GpPen, clonepen: *mut *mut GpPen) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipDeletePen"]
    pub fn GdipDeletePen(pen: *mut GpPen) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipSetPenWidth"]
    pub fn GdipSetPenWidth(pen: *mut GpPen, width: REAL) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetPenWidth"]
    pub fn GdipGetPenWidth(pen: *mut GpPen, width: *mut REAL) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipSetPenUnit"]
    pub fn GdipSetPenUnit(pen: *mut GpPen, unit: GpUnit) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetPenUnit"]
    pub fn GdipGetPenUnit(pen: *mut GpPen, unit: *mut GpUnit) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipSetPenLineCap197819"]
    pub fn GdipSetPenLineCap197819(
        pen: *mut GpPen,
        startCap: GpLineCap,
        endCap: GpLineCap,
        dashCap: GpDashCap,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipSetPenStartCap"]
    pub fn GdipSetPenStartCap(pen: *mut GpPen, startCap: GpLineCap) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipSetPenEndCap"]
    pub fn GdipSetPenEndCap(pen: *mut GpPen, endCap: GpLineCap) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipSetPenDashCap197819"]
    pub fn GdipSetPenDashCap197819(pen: *mut GpPen, dashCap: GpDashCap) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetPenStartCap"]
    pub fn GdipGetPenStartCap(pen: *mut GpPen, startCap: *mut GpLineCap) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetPenEndCap"]
    pub fn GdipGetPenEndCap(pen: *mut GpPen, endCap: *mut GpLineCap) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetPenDashCap197819"]
    pub fn GdipGetPenDashCap197819(pen: *mut GpPen, dashCap: *mut GpDashCap) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipSetPenLineJoin"]
    pub fn GdipSetPenLineJoin(pen: *mut GpPen, lineJoin: GpLineJoin) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetPenLineJoin"]
    pub fn GdipGetPenLineJoin(pen: *mut GpPen, lineJoin: *mut GpLineJoin) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipSetPenCustomStartCap"]
    pub fn GdipSetPenCustomStartCap(pen: *mut GpPen, customCap: *mut GpCustomLineCap) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetPenCustomStartCap"]
    pub fn GdipGetPenCustomStartCap(
        pen: *mut GpPen,
        customCap: *mut *mut GpCustomLineCap,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipSetPenCustomEndCap"]
    pub fn GdipSetPenCustomEndCap(pen: *mut GpPen, customCap: *mut GpCustomLineCap) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetPenCustomEndCap"]
    pub fn GdipGetPenCustomEndCap(
        pen: *mut GpPen,
        customCap: *mut *mut GpCustomLineCap,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipSetPenMiterLimit"]
    pub fn GdipSetPenMiterLimit(pen: *mut GpPen, miterLimit: REAL) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetPenMiterLimit"]
    pub fn GdipGetPenMiterLimit(pen: *mut GpPen, miterLimit: *mut REAL) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipSetPenMode"]
    pub fn GdipSetPenMode(pen: *mut GpPen, penMode: GpPenAlignment) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetPenMode"]
    pub fn GdipGetPenMode(pen: *mut GpPen, penMode: *mut GpPenAlignment) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipSetPenTransform"]
    pub fn GdipSetPenTransform(pen: *mut GpPen, matrix: *mut GpMatrix) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetPenTransform"]
    pub fn GdipGetPenTransform(pen: *mut GpPen, matrix: *mut GpMatrix) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipResetPenTransform"]
    pub fn GdipResetPenTransform(pen: *mut GpPen) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipMultiplyPenTransform"]
    pub fn GdipMultiplyPenTransform(
        pen: *mut GpPen,
        matrix: *const GpMatrix,
        order: GpMatrixOrder,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipTranslatePenTransform"]
    pub fn GdipTranslatePenTransform(
        pen: *mut GpPen,
        dx: REAL,
        dy: REAL,
        order: GpMatrixOrder,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipScalePenTransform"]
    pub fn GdipScalePenTransform(
        pen: *mut GpPen,
        sx: REAL,
        sy: REAL,
        order: GpMatrixOrder,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipRotatePenTransform"]
    pub fn GdipRotatePenTransform(pen: *mut GpPen, angle: REAL, order: GpMatrixOrder) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipSetPenColor"]
    pub fn GdipSetPenColor(pen: *mut GpPen, argb: ARGB) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetPenColor"]
    pub fn GdipGetPenColor(pen: *mut GpPen, argb: *mut ARGB) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipSetPenBrushFill"]
    pub fn GdipSetPenBrushFill(pen: *mut GpPen, brush: *mut GpBrush) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetPenBrushFill"]
    pub fn GdipGetPenBrushFill(pen: *mut GpPen, brush: *mut *mut GpBrush) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetPenFillType"]
    pub fn GdipGetPenFillType(pen: *mut GpPen, type_: *mut GpPenType) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetPenDashStyle"]
    pub fn GdipGetPenDashStyle(pen: *mut GpPen, dashstyle: *mut GpDashStyle) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipSetPenDashStyle"]
    pub fn GdipSetPenDashStyle(pen: *mut GpPen, dashstyle: GpDashStyle) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetPenDashOffset"]
    pub fn GdipGetPenDashOffset(pen: *mut GpPen, offset: *mut REAL) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipSetPenDashOffset"]
    pub fn GdipSetPenDashOffset(pen: *mut GpPen, offset: REAL) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetPenDashCount"]
    pub fn GdipGetPenDashCount(pen: *mut GpPen, count: *mut INT) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipSetPenDashArray"]
    pub fn GdipSetPenDashArray(pen: *mut GpPen, dash: *const REAL, count: INT) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetPenDashArray"]
    pub fn GdipGetPenDashArray(pen: *mut GpPen, dash: *mut REAL, count: INT) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetPenCompoundCount"]
    pub fn GdipGetPenCompoundCount(pen: *mut GpPen, count: *mut INT) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipSetPenCompoundArray"]
    pub fn GdipSetPenCompoundArray(pen: *mut GpPen, dash: *const REAL, count: INT) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetPenCompoundArray"]
    pub fn GdipGetPenCompoundArray(pen: *mut GpPen, dash: *mut REAL, count: INT) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipCreateCustomLineCap"]
    pub fn GdipCreateCustomLineCap(
        fillPath: *mut GpPath,
        strokePath: *mut GpPath,
        baseCap: GpLineCap,
        baseInset: REAL,
        customCap: *mut *mut GpCustomLineCap,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipDeleteCustomLineCap"]
    pub fn GdipDeleteCustomLineCap(customCap: *mut GpCustomLineCap) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipCloneCustomLineCap"]
    pub fn GdipCloneCustomLineCap(
        customCap: *mut GpCustomLineCap,
        clonedCap: *mut *mut GpCustomLineCap,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetCustomLineCapType"]
    pub fn GdipGetCustomLineCapType(
        customCap: *mut GpCustomLineCap,
        capType: *mut CustomLineCapType,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipSetCustomLineCapStrokeCaps"]
    pub fn GdipSetCustomLineCapStrokeCaps(
        customCap: *mut GpCustomLineCap,
        startCap: GpLineCap,
        endCap: GpLineCap,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetCustomLineCapStrokeCaps"]
    pub fn GdipGetCustomLineCapStrokeCaps(
        customCap: *mut GpCustomLineCap,
        startCap: *mut GpLineCap,
        endCap: *mut GpLineCap,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipSetCustomLineCapStrokeJoin"]
    pub fn GdipSetCustomLineCapStrokeJoin(
        customCap: *mut GpCustomLineCap,
        lineJoin: GpLineJoin,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetCustomLineCapStrokeJoin"]
    pub fn GdipGetCustomLineCapStrokeJoin(
        customCap: *mut GpCustomLineCap,
        lineJoin: *mut GpLineJoin,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipSetCustomLineCapBaseCap"]
    pub fn GdipSetCustomLineCapBaseCap(
        customCap: *mut GpCustomLineCap,
        baseCap: GpLineCap,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetCustomLineCapBaseCap"]
    pub fn GdipGetCustomLineCapBaseCap(
        customCap: *mut GpCustomLineCap,
        baseCap: *mut GpLineCap,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipSetCustomLineCapBaseInset"]
    pub fn GdipSetCustomLineCapBaseInset(customCap: *mut GpCustomLineCap, inset: REAL) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetCustomLineCapBaseInset"]
    pub fn GdipGetCustomLineCapBaseInset(
        customCap: *mut GpCustomLineCap,
        inset: *mut REAL,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipSetCustomLineCapWidthScale"]
    pub fn GdipSetCustomLineCapWidthScale(
        customCap: *mut GpCustomLineCap,
        widthScale: REAL,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetCustomLineCapWidthScale"]
    pub fn GdipGetCustomLineCapWidthScale(
        customCap: *mut GpCustomLineCap,
        widthScale: *mut REAL,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipCreateAdjustableArrowCap"]
    pub fn GdipCreateAdjustableArrowCap(
        height: REAL,
        width: REAL,
        isFilled: BOOL,
        cap: *mut *mut GpAdjustableArrowCap,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipSetAdjustableArrowCapHeight"]
    pub fn GdipSetAdjustableArrowCapHeight(
        cap: *mut GpAdjustableArrowCap,
        height: REAL,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetAdjustableArrowCapHeight"]
    pub fn GdipGetAdjustableArrowCapHeight(
        cap: *mut GpAdjustableArrowCap,
        height: *mut REAL,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipSetAdjustableArrowCapWidth"]
    pub fn GdipSetAdjustableArrowCapWidth(cap: *mut GpAdjustableArrowCap, width: REAL) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetAdjustableArrowCapWidth"]
    pub fn GdipGetAdjustableArrowCapWidth(
        cap: *mut GpAdjustableArrowCap,
        width: *mut REAL,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipSetAdjustableArrowCapMiddleInset"]
    pub fn GdipSetAdjustableArrowCapMiddleInset(
        cap: *mut GpAdjustableArrowCap,
        middleInset: REAL,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetAdjustableArrowCapMiddleInset"]
    pub fn GdipGetAdjustableArrowCapMiddleInset(
        cap: *mut GpAdjustableArrowCap,
        middleInset: *mut REAL,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipSetAdjustableArrowCapFillState"]
    pub fn GdipSetAdjustableArrowCapFillState(
        cap: *mut GpAdjustableArrowCap,
        fillState: BOOL,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetAdjustableArrowCapFillState"]
    pub fn GdipGetAdjustableArrowCapFillState(
        cap: *mut GpAdjustableArrowCap,
        fillState: *mut BOOL,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipLoadImageFromStream"]
    pub fn GdipLoadImageFromStream(stream: *mut IStream, image: *mut *mut GpImage) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipLoadImageFromFile"]
    pub fn GdipLoadImageFromFile(filename: *const WCHAR, image: *mut *mut GpImage) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipLoadImageFromStreamICM"]
    pub fn GdipLoadImageFromStreamICM(stream: *mut IStream, image: *mut *mut GpImage) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipLoadImageFromFileICM"]
    pub fn GdipLoadImageFromFileICM(filename: *const WCHAR, image: *mut *mut GpImage) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipCloneImage"]
    pub fn GdipCloneImage(image: *mut GpImage, cloneImage: *mut *mut GpImage) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipDisposeImage"]
    pub fn GdipDisposeImage(image: *mut GpImage) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipSaveImageToFile"]
    pub fn GdipSaveImageToFile(
        image: *mut GpImage,
        filename: *const WCHAR,
        clsidEncoder: *const CLSID,
        encoderParams: *const EncoderParameters,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipSaveImageToStream"]
    pub fn GdipSaveImageToStream(
        image: *mut GpImage,
        stream: *mut IStream,
        clsidEncoder: *const CLSID,
        encoderParams: *const EncoderParameters,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipSaveAdd"]
    pub fn GdipSaveAdd(image: *mut GpImage, encoderParams: *const EncoderParameters) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipSaveAddImage"]
    pub fn GdipSaveAddImage(
        image: *mut GpImage,
        newImage: *mut GpImage,
        encoderParams: *const EncoderParameters,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetImageGraphicsContext"]
    pub fn GdipGetImageGraphicsContext(
        image: *mut GpImage,
        graphics: *mut *mut GpGraphics,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetImageBounds"]
    pub fn GdipGetImageBounds(
        image: *mut GpImage,
        srcRect: *mut GpRectF,
        srcUnit: *mut GpUnit,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetImageDimension"]
    pub fn GdipGetImageDimension(
        image: *mut GpImage,
        width: *mut REAL,
        height: *mut REAL,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetImageType"]
    pub fn GdipGetImageType(image: *mut GpImage, type_: *mut ImageType) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetImageWidth"]
    pub fn GdipGetImageWidth(image: *mut GpImage, width: *mut UINT) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetImageHeight"]
    pub fn GdipGetImageHeight(image: *mut GpImage, height: *mut UINT) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetImageHorizontalResolution"]
    pub fn GdipGetImageHorizontalResolution(image: *mut GpImage, resolution: *mut REAL)
        -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetImageVerticalResolution"]
    pub fn GdipGetImageVerticalResolution(image: *mut GpImage, resolution: *mut REAL) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetImageFlags"]
    pub fn GdipGetImageFlags(image: *mut GpImage, flags: *mut UINT) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetImageRawFormat"]
    pub fn GdipGetImageRawFormat(image: *mut GpImage, format: *mut GUID) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetImagePixelFormat"]
    pub fn GdipGetImagePixelFormat(image: *mut GpImage, format: *mut PixelFormat) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetImageThumbnail"]
    pub fn GdipGetImageThumbnail(
        image: *mut GpImage,
        thumbWidth: UINT,
        thumbHeight: UINT,
        thumbImage: *mut *mut GpImage,
        callback: GetThumbnailImageAbort,
        callbackData: *mut c_void,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetEncoderParameterListSize"]
    pub fn GdipGetEncoderParameterListSize(
        image: *mut GpImage,
        clsidEncoder: *const CLSID,
        size: *mut UINT,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetEncoderParameterList"]
    pub fn GdipGetEncoderParameterList(
        image: *mut GpImage,
        clsidEncoder: *const CLSID,
        size: UINT,
        buffer: *mut EncoderParameters,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipImageGetFrameDimensionsCount"]
    pub fn GdipImageGetFrameDimensionsCount(image: *mut GpImage, count: *mut UINT) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipImageGetFrameDimensionsList"]
    pub fn GdipImageGetFrameDimensionsList(
        image: *mut GpImage,
        dimensionIDs: *mut GUID,
        count: UINT,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipImageGetFrameCount"]
    pub fn GdipImageGetFrameCount(
        image: *mut GpImage,
        dimensionID: *const GUID,
        count: *mut UINT,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipImageSelectActiveFrame"]
    pub fn GdipImageSelectActiveFrame(
        image: *mut GpImage,
        dimensionID: *const GUID,
        frameIndex: UINT,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipImageRotateFlip"]
    pub fn GdipImageRotateFlip(image: *mut GpImage, rfType: RotateFlipType) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetImagePalette"]
    pub fn GdipGetImagePalette(
        image: *mut GpImage,
        palette: *mut ColorPalette,
        size: INT,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipSetImagePalette"]
    pub fn GdipSetImagePalette(image: *mut GpImage, palette: *const ColorPalette) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetImagePaletteSize"]
    pub fn GdipGetImagePaletteSize(image: *mut GpImage, size: *mut INT) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetPropertyCount"]
    pub fn GdipGetPropertyCount(image: *mut GpImage, numOfProperty: *mut UINT) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetPropertyIdList"]
    pub fn GdipGetPropertyIdList(
        image: *mut GpImage,
        numOfProperty: UINT,
        list: *mut PROPID,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetPropertyItemSize"]
    pub fn GdipGetPropertyItemSize(
        image: *mut GpImage,
        propId: PROPID,
        size: *mut UINT,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetPropertyItem"]
    pub fn GdipGetPropertyItem(
        image: *mut GpImage,
        propId: PROPID,
        propSize: UINT,
        buffer: *mut PropertyItem,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetPropertySize"]
    pub fn GdipGetPropertySize(
        image: *mut GpImage,
        totalBufferSize: *mut UINT,
        numProperties: *mut UINT,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetAllPropertyItems"]
    pub fn GdipGetAllPropertyItems(
        image: *mut GpImage,
        totalBufferSize: UINT,
        numProperties: UINT,
        allItems: *mut PropertyItem,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipRemovePropertyItem"]
    pub fn GdipRemovePropertyItem(image: *mut GpImage, propId: PROPID) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipSetPropertyItem"]
    pub fn GdipSetPropertyItem(image: *mut GpImage, item: *const PropertyItem) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipImageForceValidation"]
    pub fn GdipImageForceValidation(image: *mut GpImage) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipCreateBitmapFromStream"]
    pub fn GdipCreateBitmapFromStream(stream: *mut IStream, bitmap: *mut *mut GpBitmap)
        -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipCreateBitmapFromFile"]
    pub fn GdipCreateBitmapFromFile(filename: *const WCHAR, bitmap: *mut *mut GpBitmap)
        -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipCreateBitmapFromStreamICM"]
    pub fn GdipCreateBitmapFromStreamICM(
        stream: *mut IStream,
        bitmap: *mut *mut GpBitmap,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipCreateBitmapFromFileICM"]
    pub fn GdipCreateBitmapFromFileICM(
        filename: *const WCHAR,
        bitmap: *mut *mut GpBitmap,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipCreateBitmapFromScan0"]
    pub fn GdipCreateBitmapFromScan0(
        width: INT,
        height: INT,
        stride: INT,
        format: PixelFormat,
        scan0: *mut BYTE,
        bitmap: *mut *mut GpBitmap,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipCreateBitmapFromGraphics"]
    pub fn GdipCreateBitmapFromGraphics(
        width: INT,
        height: INT,
        target: *mut GpGraphics,
        bitmap: *mut *mut GpBitmap,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipCreateBitmapFromDirectDrawSurface"]
    pub fn GdipCreateBitmapFromDirectDrawSurface(
        surface: *mut IDirectDrawSurface7,
        bitmap: *mut *mut GpBitmap,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipCreateBitmapFromGdiDib"]
    pub fn GdipCreateBitmapFromGdiDib(
        gdiBitmapInfo: *const BITMAPINFO,
        gdiBitmapData: *mut c_void,
        bitmap: *mut *mut GpBitmap,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipCreateBitmapFromHBITMAP"]
    pub fn GdipCreateBitmapFromHBITMAP(
        hbm: HBITMAP,
        hpal: HPALETTE,
        bitmap: *mut *mut GpBitmap,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipCreateHBITMAPFromBitmap"]
    pub fn GdipCreateHBITMAPFromBitmap(
        bitmap: *mut GpBitmap,
        hbmReturn: *mut HBITMAP,
        background: ARGB,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipCreateBitmapFromHICON"]
    pub fn GdipCreateBitmapFromHICON(hicon: HICON, bitmap: *mut *mut GpBitmap) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipCreateHICONFromBitmap"]
    pub fn GdipCreateHICONFromBitmap(bitmap: *mut GpBitmap, hbmReturn: *mut HICON) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipCreateBitmapFromResource"]
    pub fn GdipCreateBitmapFromResource(
        hInstance: HINSTANCE,
        lpBitmapName: *const WCHAR,
        bitmap: *mut *mut GpBitmap,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipCloneBitmapArea"]
    pub fn GdipCloneBitmapArea(
        x: REAL,
        y: REAL,
        width: REAL,
        height: REAL,
        format: PixelFormat,
        srcBitmap: *mut GpBitmap,
        dstBitmap: *mut *mut GpBitmap,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipCloneBitmapAreaI"]
    pub fn GdipCloneBitmapAreaI(
        x: INT,
        y: INT,
        width: INT,
        height: INT,
        format: PixelFormat,
        srcBitmap: *mut GpBitmap,
        dstBitmap: *mut *mut GpBitmap,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipBitmapLockBits"]
    pub fn GdipBitmapLockBits(
        bitmap: *mut GpBitmap,
        rect: *const GpRect,
        flags: UINT,
        format: PixelFormat,
        lockedBitmapData: *mut BitmapData,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipBitmapUnlockBits"]
    pub fn GdipBitmapUnlockBits(
        bitmap: *mut GpBitmap,
        lockedBitmapData: *mut BitmapData,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipBitmapGetPixel"]
    pub fn GdipBitmapGetPixel(bitmap: *mut GpBitmap, x: INT, y: INT, color: *mut ARGB) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipBitmapSetPixel"]
    pub fn GdipBitmapSetPixel(bitmap: *mut GpBitmap, x: INT, y: INT, color: ARGB) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipBitmapSetResolution"]
    pub fn GdipBitmapSetResolution(bitmap: *mut GpBitmap, xdpi: REAL, ydpi: REAL) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipCreateImageAttributes"]
    pub fn GdipCreateImageAttributes(imageattr: *mut *mut GpImageAttributes) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipCloneImageAttributes"]
    pub fn GdipCloneImageAttributes(
        imageattr: *const GpImageAttributes,
        cloneImageattr: *mut *mut GpImageAttributes,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipDisposeImageAttributes"]
    pub fn GdipDisposeImageAttributes(imageattr: *mut GpImageAttributes) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipSetImageAttributesToIdentity"]
    pub fn GdipSetImageAttributesToIdentity(
        imageattr: *mut GpImageAttributes,
        type_: ColorAdjustType,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipResetImageAttributes"]
    pub fn GdipResetImageAttributes(
        imageattr: *mut GpImageAttributes,
        type_: ColorAdjustType,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipSetImageAttributesColorMatrix"]
    pub fn GdipSetImageAttributesColorMatrix(
        imageattr: *mut GpImageAttributes,
        type_: ColorAdjustType,
        enableFlag: BOOL,
        colorMatrix: *const ColorMatrix,
        grayMatrix: *const ColorMatrix,
        flags: ColorMatrixFlags,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipSetImageAttributesThreshold"]
    pub fn GdipSetImageAttributesThreshold(
        imageattr: *mut GpImageAttributes,
        type_: ColorAdjustType,
        enableFlag: BOOL,
        threshold: REAL,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipSetImageAttributesGamma"]
    pub fn GdipSetImageAttributesGamma(
        imageattr: *mut GpImageAttributes,
        type_: ColorAdjustType,
        enableFlag: BOOL,
        gamma: REAL,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipSetImageAttributesNoOp"]
    pub fn GdipSetImageAttributesNoOp(
        imageattr: *mut GpImageAttributes,
        type_: ColorAdjustType,
        enableFlag: BOOL,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipSetImageAttributesColorKeys"]
    pub fn GdipSetImageAttributesColorKeys(
        imageattr: *mut GpImageAttributes,
        type_: ColorAdjustType,
        enableFlag: BOOL,
        colorLow: ARGB,
        colorHigh: ARGB,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipSetImageAttributesOutputChannel"]
    pub fn GdipSetImageAttributesOutputChannel(
        imageattr: *mut GpImageAttributes,
        type_: ColorAdjustType,
        enableFlag: BOOL,
        channelFlags: ColorChannelFlags,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipSetImageAttributesOutputChannelColorProfile"]
    pub fn GdipSetImageAttributesOutputChannelColorProfile(
        imageattr: *mut GpImageAttributes,
        type_: ColorAdjustType,
        enableFlag: BOOL,
        colorProfileFilename: *const WCHAR,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipSetImageAttributesRemapTable"]
    pub fn GdipSetImageAttributesRemapTable(
        imageattr: *mut GpImageAttributes,
        type_: ColorAdjustType,
        enableFlag: BOOL,
        mapSize: UINT,
        map: *const ColorMap,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipSetImageAttributesWrapMode"]
    pub fn GdipSetImageAttributesWrapMode(
        imageAttr: *mut GpImageAttributes,
        wrap: WrapMode,
        argb: ARGB,
        clamp: BOOL,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipSetImageAttributesICMMode"]
    pub fn GdipSetImageAttributesICMMode(imageAttr: *mut GpImageAttributes, on: BOOL) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetImageAttributesAdjustedPalette"]
    pub fn GdipGetImageAttributesAdjustedPalette(
        imageAttr: *mut GpImageAttributes,
        colorPalette: *mut ColorPalette,
        colorAdjustType: ColorAdjustType,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipFlush"]
    pub fn GdipFlush(graphics: *mut GpGraphics, intention: GpFlushIntention) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipCreateFromHDC"]
    pub fn GdipCreateFromHDC(hdc: HDC, graphics: *mut *mut GpGraphics) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipCreateFromHDC2"]
    pub fn GdipCreateFromHDC2(
        hdc: HDC,
        hDevice: HANDLE,
        graphics: *mut *mut GpGraphics,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipCreateFromHWND"]
    pub fn GdipCreateFromHWND(hwnd: HWND, graphics: *mut *mut GpGraphics) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipCreateFromHWNDICM"]
    pub fn GdipCreateFromHWNDICM(hwnd: HWND, graphics: *mut *mut GpGraphics) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipDeleteGraphics"]
    pub fn GdipDeleteGraphics(graphics: *mut GpGraphics) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetDC"]
    pub fn GdipGetDC(graphics: *mut GpGraphics, hdc: *mut HDC) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipReleaseDC"]
    pub fn GdipReleaseDC(graphics: *mut GpGraphics, hdc: HDC) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipSetCompositingMode"]
    pub fn GdipSetCompositingMode(
        graphics: *mut GpGraphics,
        compositingMode: CompositingMode,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetCompositingMode"]
    pub fn GdipGetCompositingMode(
        graphics: *mut GpGraphics,
        compositingMode: *mut CompositingMode,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipSetRenderingOrigin"]
    pub fn GdipSetRenderingOrigin(graphics: *mut GpGraphics, x: INT, y: INT) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetRenderingOrigin"]
    pub fn GdipGetRenderingOrigin(graphics: *mut GpGraphics, x: *mut INT, y: *mut INT) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipSetCompositingQuality"]
    pub fn GdipSetCompositingQuality(
        graphics: *mut GpGraphics,
        compositingQuality: CompositingQuality,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetCompositingQuality"]
    pub fn GdipGetCompositingQuality(
        graphics: *mut GpGraphics,
        compositingQuality: *mut CompositingQuality,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipSetSmoothingMode"]
    pub fn GdipSetSmoothingMode(
        graphics: *mut GpGraphics,
        smoothingMode: SmoothingMode,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetSmoothingMode"]
    pub fn GdipGetSmoothingMode(
        graphics: *mut GpGraphics,
        smoothingMode: *mut SmoothingMode,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipSetPixelOffsetMode"]
    pub fn GdipSetPixelOffsetMode(
        graphics: *mut GpGraphics,
        pixelOffsetMode: PixelOffsetMode,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetPixelOffsetMode"]
    pub fn GdipGetPixelOffsetMode(
        graphics: *mut GpGraphics,
        pixelOffsetMode: *mut PixelOffsetMode,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipSetTextRenderingHint"]
    pub fn GdipSetTextRenderingHint(graphics: *mut GpGraphics, mode: TextRenderingHint)
        -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetTextRenderingHint"]
    pub fn GdipGetTextRenderingHint(
        graphics: *mut GpGraphics,
        mode: *mut TextRenderingHint,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipSetTextContrast"]
    pub fn GdipSetTextContrast(graphics: *mut GpGraphics, contrast: UINT) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetTextContrast"]
    pub fn GdipGetTextContrast(graphics: *mut GpGraphics, contrast: *mut UINT) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipSetInterpolationMode"]
    pub fn GdipSetInterpolationMode(
        graphics: *mut GpGraphics,
        interpolationMode: InterpolationMode,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetInterpolationMode"]
    pub fn GdipGetInterpolationMode(
        graphics: *mut GpGraphics,
        interpolationMode: *mut InterpolationMode,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipSetWorldTransform"]
    pub fn GdipSetWorldTransform(graphics: *mut GpGraphics, matrix: *mut GpMatrix) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipResetWorldTransform"]
    pub fn GdipResetWorldTransform(graphics: *mut GpGraphics) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipMultiplyWorldTransform"]
    pub fn GdipMultiplyWorldTransform(
        graphics: *mut GpGraphics,
        matrix: *const GpMatrix,
        order: GpMatrixOrder,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipTranslateWorldTransform"]
    pub fn GdipTranslateWorldTransform(
        graphics: *mut GpGraphics,
        dx: REAL,
        dy: REAL,
        order: GpMatrixOrder,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipScaleWorldTransform"]
    pub fn GdipScaleWorldTransform(
        graphics: *mut GpGraphics,
        sx: REAL,
        sy: REAL,
        order: GpMatrixOrder,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipRotateWorldTransform"]
    pub fn GdipRotateWorldTransform(
        graphics: *mut GpGraphics,
        angle: REAL,
        order: GpMatrixOrder,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetWorldTransform"]
    pub fn GdipGetWorldTransform(graphics: *mut GpGraphics, matrix: *mut GpMatrix) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipResetPageTransform"]
    pub fn GdipResetPageTransform(graphics: *mut GpGraphics) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetPageUnit"]
    pub fn GdipGetPageUnit(graphics: *mut GpGraphics, unit: *mut GpUnit) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetPageScale"]
    pub fn GdipGetPageScale(graphics: *mut GpGraphics, scale: *mut REAL) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipSetPageUnit"]
    pub fn GdipSetPageUnit(graphics: *mut GpGraphics, unit: GpUnit) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipSetPageScale"]
    pub fn GdipSetPageScale(graphics: *mut GpGraphics, scale: REAL) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetDpiX"]
    pub fn GdipGetDpiX(graphics: *mut GpGraphics, dpi: *mut REAL) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetDpiY"]
    pub fn GdipGetDpiY(graphics: *mut GpGraphics, dpi: *mut REAL) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipTransformPoints"]
    pub fn GdipTransformPoints(
        graphics: *mut GpGraphics,
        destSpace: GpCoordinateSpace,
        srcSpace: GpCoordinateSpace,
        points: *mut GpPointF,
        count: INT,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipTransformPointsI"]
    pub fn GdipTransformPointsI(
        graphics: *mut GpGraphics,
        destSpace: GpCoordinateSpace,
        srcSpace: GpCoordinateSpace,
        points: *mut GpPoint,
        count: INT,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetNearestColor"]
    pub fn GdipGetNearestColor(graphics: *mut GpGraphics, argb: *mut ARGB) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipCreateHalftonePalette"]
    pub fn GdipCreateHalftonePalette() -> HPALETTE;
}
extern "C" {
    #[link_name = "\u{1}GdipDrawLine"]
    pub fn GdipDrawLine(
        graphics: *mut GpGraphics,
        pen: *mut GpPen,
        x1: REAL,
        y1: REAL,
        x2: REAL,
        y2: REAL,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipDrawLineI"]
    pub fn GdipDrawLineI(
        graphics: *mut GpGraphics,
        pen: *mut GpPen,
        x1: INT,
        y1: INT,
        x2: INT,
        y2: INT,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipDrawLines"]
    pub fn GdipDrawLines(
        graphics: *mut GpGraphics,
        pen: *mut GpPen,
        points: *const GpPointF,
        count: INT,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipDrawLinesI"]
    pub fn GdipDrawLinesI(
        graphics: *mut GpGraphics,
        pen: *mut GpPen,
        points: *const GpPoint,
        count: INT,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipDrawArc"]
    pub fn GdipDrawArc(
        graphics: *mut GpGraphics,
        pen: *mut GpPen,
        x: REAL,
        y: REAL,
        width: REAL,
        height: REAL,
        startAngle: REAL,
        sweepAngle: REAL,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipDrawArcI"]
    pub fn GdipDrawArcI(
        graphics: *mut GpGraphics,
        pen: *mut GpPen,
        x: INT,
        y: INT,
        width: INT,
        height: INT,
        startAngle: REAL,
        sweepAngle: REAL,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipDrawBezier"]
    pub fn GdipDrawBezier(
        graphics: *mut GpGraphics,
        pen: *mut GpPen,
        x1: REAL,
        y1: REAL,
        x2: REAL,
        y2: REAL,
        x3: REAL,
        y3: REAL,
        x4: REAL,
        y4: REAL,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipDrawBezierI"]
    pub fn GdipDrawBezierI(
        graphics: *mut GpGraphics,
        pen: *mut GpPen,
        x1: INT,
        y1: INT,
        x2: INT,
        y2: INT,
        x3: INT,
        y3: INT,
        x4: INT,
        y4: INT,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipDrawBeziers"]
    pub fn GdipDrawBeziers(
        graphics: *mut GpGraphics,
        pen: *mut GpPen,
        points: *const GpPointF,
        count: INT,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipDrawBeziersI"]
    pub fn GdipDrawBeziersI(
        graphics: *mut GpGraphics,
        pen: *mut GpPen,
        points: *const GpPoint,
        count: INT,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipDrawRectangle"]
    pub fn GdipDrawRectangle(
        graphics: *mut GpGraphics,
        pen: *mut GpPen,
        x: REAL,
        y: REAL,
        width: REAL,
        height: REAL,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipDrawRectangleI"]
    pub fn GdipDrawRectangleI(
        graphics: *mut GpGraphics,
        pen: *mut GpPen,
        x: INT,
        y: INT,
        width: INT,
        height: INT,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipDrawRectangles"]
    pub fn GdipDrawRectangles(
        graphics: *mut GpGraphics,
        pen: *mut GpPen,
        rects: *const GpRectF,
        count: INT,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipDrawRectanglesI"]
    pub fn GdipDrawRectanglesI(
        graphics: *mut GpGraphics,
        pen: *mut GpPen,
        rects: *const GpRect,
        count: INT,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipDrawEllipse"]
    pub fn GdipDrawEllipse(
        graphics: *mut GpGraphics,
        pen: *mut GpPen,
        x: REAL,
        y: REAL,
        width: REAL,
        height: REAL,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipDrawEllipseI"]
    pub fn GdipDrawEllipseI(
        graphics: *mut GpGraphics,
        pen: *mut GpPen,
        x: INT,
        y: INT,
        width: INT,
        height: INT,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipDrawPie"]
    pub fn GdipDrawPie(
        graphics: *mut GpGraphics,
        pen: *mut GpPen,
        x: REAL,
        y: REAL,
        width: REAL,
        height: REAL,
        startAngle: REAL,
        sweepAngle: REAL,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipDrawPieI"]
    pub fn GdipDrawPieI(
        graphics: *mut GpGraphics,
        pen: *mut GpPen,
        x: INT,
        y: INT,
        width: INT,
        height: INT,
        startAngle: REAL,
        sweepAngle: REAL,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipDrawPolygon"]
    pub fn GdipDrawPolygon(
        graphics: *mut GpGraphics,
        pen: *mut GpPen,
        points: *const GpPointF,
        count: INT,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipDrawPolygonI"]
    pub fn GdipDrawPolygonI(
        graphics: *mut GpGraphics,
        pen: *mut GpPen,
        points: *const GpPoint,
        count: INT,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipDrawPath"]
    pub fn GdipDrawPath(graphics: *mut GpGraphics, pen: *mut GpPen, path: *mut GpPath) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipDrawCurve"]
    pub fn GdipDrawCurve(
        graphics: *mut GpGraphics,
        pen: *mut GpPen,
        points: *const GpPointF,
        count: INT,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipDrawCurveI"]
    pub fn GdipDrawCurveI(
        graphics: *mut GpGraphics,
        pen: *mut GpPen,
        points: *const GpPoint,
        count: INT,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipDrawCurve2"]
    pub fn GdipDrawCurve2(
        graphics: *mut GpGraphics,
        pen: *mut GpPen,
        points: *const GpPointF,
        count: INT,
        tension: REAL,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipDrawCurve2I"]
    pub fn GdipDrawCurve2I(
        graphics: *mut GpGraphics,
        pen: *mut GpPen,
        points: *const GpPoint,
        count: INT,
        tension: REAL,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipDrawCurve3"]
    pub fn GdipDrawCurve3(
        graphics: *mut GpGraphics,
        pen: *mut GpPen,
        points: *const GpPointF,
        count: INT,
        offset: INT,
        numberOfSegments: INT,
        tension: REAL,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipDrawCurve3I"]
    pub fn GdipDrawCurve3I(
        graphics: *mut GpGraphics,
        pen: *mut GpPen,
        points: *const GpPoint,
        count: INT,
        offset: INT,
        numberOfSegments: INT,
        tension: REAL,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipDrawClosedCurve"]
    pub fn GdipDrawClosedCurve(
        graphics: *mut GpGraphics,
        pen: *mut GpPen,
        points: *const GpPointF,
        count: INT,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipDrawClosedCurveI"]
    pub fn GdipDrawClosedCurveI(
        graphics: *mut GpGraphics,
        pen: *mut GpPen,
        points: *const GpPoint,
        count: INT,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipDrawClosedCurve2"]
    pub fn GdipDrawClosedCurve2(
        graphics: *mut GpGraphics,
        pen: *mut GpPen,
        points: *const GpPointF,
        count: INT,
        tension: REAL,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipDrawClosedCurve2I"]
    pub fn GdipDrawClosedCurve2I(
        graphics: *mut GpGraphics,
        pen: *mut GpPen,
        points: *const GpPoint,
        count: INT,
        tension: REAL,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGraphicsClear"]
    pub fn GdipGraphicsClear(graphics: *mut GpGraphics, color: ARGB) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipFillRectangle"]
    pub fn GdipFillRectangle(
        graphics: *mut GpGraphics,
        brush: *mut GpBrush,
        x: REAL,
        y: REAL,
        width: REAL,
        height: REAL,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipFillRectangleI"]
    pub fn GdipFillRectangleI(
        graphics: *mut GpGraphics,
        brush: *mut GpBrush,
        x: INT,
        y: INT,
        width: INT,
        height: INT,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipFillRectangles"]
    pub fn GdipFillRectangles(
        graphics: *mut GpGraphics,
        brush: *mut GpBrush,
        rects: *const GpRectF,
        count: INT,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipFillRectanglesI"]
    pub fn GdipFillRectanglesI(
        graphics: *mut GpGraphics,
        brush: *mut GpBrush,
        rects: *const GpRect,
        count: INT,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipFillPolygon"]
    pub fn GdipFillPolygon(
        graphics: *mut GpGraphics,
        brush: *mut GpBrush,
        points: *const GpPointF,
        count: INT,
        fillMode: GpFillMode,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipFillPolygonI"]
    pub fn GdipFillPolygonI(
        graphics: *mut GpGraphics,
        brush: *mut GpBrush,
        points: *const GpPoint,
        count: INT,
        fillMode: GpFillMode,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipFillPolygon2"]
    pub fn GdipFillPolygon2(
        graphics: *mut GpGraphics,
        brush: *mut GpBrush,
        points: *const GpPointF,
        count: INT,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipFillPolygon2I"]
    pub fn GdipFillPolygon2I(
        graphics: *mut GpGraphics,
        brush: *mut GpBrush,
        points: *const GpPoint,
        count: INT,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipFillEllipse"]
    pub fn GdipFillEllipse(
        graphics: *mut GpGraphics,
        brush: *mut GpBrush,
        x: REAL,
        y: REAL,
        width: REAL,
        height: REAL,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipFillEllipseI"]
    pub fn GdipFillEllipseI(
        graphics: *mut GpGraphics,
        brush: *mut GpBrush,
        x: INT,
        y: INT,
        width: INT,
        height: INT,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipFillPie"]
    pub fn GdipFillPie(
        graphics: *mut GpGraphics,
        brush: *mut GpBrush,
        x: REAL,
        y: REAL,
        width: REAL,
        height: REAL,
        startAngle: REAL,
        sweepAngle: REAL,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipFillPieI"]
    pub fn GdipFillPieI(
        graphics: *mut GpGraphics,
        brush: *mut GpBrush,
        x: INT,
        y: INT,
        width: INT,
        height: INT,
        startAngle: REAL,
        sweepAngle: REAL,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipFillPath"]
    pub fn GdipFillPath(
        graphics: *mut GpGraphics,
        brush: *mut GpBrush,
        path: *mut GpPath,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipFillClosedCurve"]
    pub fn GdipFillClosedCurve(
        graphics: *mut GpGraphics,
        brush: *mut GpBrush,
        points: *const GpPointF,
        count: INT,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipFillClosedCurveI"]
    pub fn GdipFillClosedCurveI(
        graphics: *mut GpGraphics,
        brush: *mut GpBrush,
        points: *const GpPoint,
        count: INT,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipFillClosedCurve2"]
    pub fn GdipFillClosedCurve2(
        graphics: *mut GpGraphics,
        brush: *mut GpBrush,
        points: *const GpPointF,
        count: INT,
        tension: REAL,
        fillMode: GpFillMode,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipFillClosedCurve2I"]
    pub fn GdipFillClosedCurve2I(
        graphics: *mut GpGraphics,
        brush: *mut GpBrush,
        points: *const GpPoint,
        count: INT,
        tension: REAL,
        fillMode: GpFillMode,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipFillRegion"]
    pub fn GdipFillRegion(
        graphics: *mut GpGraphics,
        brush: *mut GpBrush,
        region: *mut GpRegion,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipDrawImage"]
    pub fn GdipDrawImage(
        graphics: *mut GpGraphics,
        image: *mut GpImage,
        x: REAL,
        y: REAL,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipDrawImageI"]
    pub fn GdipDrawImageI(
        graphics: *mut GpGraphics,
        image: *mut GpImage,
        x: INT,
        y: INT,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipDrawImageRect"]
    pub fn GdipDrawImageRect(
        graphics: *mut GpGraphics,
        image: *mut GpImage,
        x: REAL,
        y: REAL,
        width: REAL,
        height: REAL,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipDrawImageRectI"]
    pub fn GdipDrawImageRectI(
        graphics: *mut GpGraphics,
        image: *mut GpImage,
        x: INT,
        y: INT,
        width: INT,
        height: INT,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipDrawImagePoints"]
    pub fn GdipDrawImagePoints(
        graphics: *mut GpGraphics,
        image: *mut GpImage,
        dstpoints: *const GpPointF,
        count: INT,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipDrawImagePointsI"]
    pub fn GdipDrawImagePointsI(
        graphics: *mut GpGraphics,
        image: *mut GpImage,
        dstpoints: *const GpPoint,
        count: INT,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipDrawImagePointRect"]
    pub fn GdipDrawImagePointRect(
        graphics: *mut GpGraphics,
        image: *mut GpImage,
        x: REAL,
        y: REAL,
        srcx: REAL,
        srcy: REAL,
        srcwidth: REAL,
        srcheight: REAL,
        srcUnit: GpUnit,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipDrawImagePointRectI"]
    pub fn GdipDrawImagePointRectI(
        graphics: *mut GpGraphics,
        image: *mut GpImage,
        x: INT,
        y: INT,
        srcx: INT,
        srcy: INT,
        srcwidth: INT,
        srcheight: INT,
        srcUnit: GpUnit,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipDrawImageRectRect"]
    pub fn GdipDrawImageRectRect(
        graphics: *mut GpGraphics,
        image: *mut GpImage,
        dstx: REAL,
        dsty: REAL,
        dstwidth: REAL,
        dstheight: REAL,
        srcx: REAL,
        srcy: REAL,
        srcwidth: REAL,
        srcheight: REAL,
        srcUnit: GpUnit,
        imageAttributes: *const GpImageAttributes,
        callback: DrawImageAbort,
        callbackData: *mut c_void,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipDrawImageRectRectI"]
    pub fn GdipDrawImageRectRectI(
        graphics: *mut GpGraphics,
        image: *mut GpImage,
        dstx: INT,
        dsty: INT,
        dstwidth: INT,
        dstheight: INT,
        srcx: INT,
        srcy: INT,
        srcwidth: INT,
        srcheight: INT,
        srcUnit: GpUnit,
        imageAttributes: *const GpImageAttributes,
        callback: DrawImageAbort,
        callbackData: *mut c_void,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipDrawImagePointsRect"]
    pub fn GdipDrawImagePointsRect(
        graphics: *mut GpGraphics,
        image: *mut GpImage,
        points: *const GpPointF,
        count: INT,
        srcx: REAL,
        srcy: REAL,
        srcwidth: REAL,
        srcheight: REAL,
        srcUnit: GpUnit,
        imageAttributes: *const GpImageAttributes,
        callback: DrawImageAbort,
        callbackData: *mut c_void,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipDrawImagePointsRectI"]
    pub fn GdipDrawImagePointsRectI(
        graphics: *mut GpGraphics,
        image: *mut GpImage,
        points: *const GpPoint,
        count: INT,
        srcx: INT,
        srcy: INT,
        srcwidth: INT,
        srcheight: INT,
        srcUnit: GpUnit,
        imageAttributes: *const GpImageAttributes,
        callback: DrawImageAbort,
        callbackData: *mut c_void,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipEnumerateMetafileDestPoint"]
    pub fn GdipEnumerateMetafileDestPoint(
        graphics: *mut GpGraphics,
        metafile: *const GpMetafile,
        destPoint: *const PointF,
        callback: EnumerateMetafileProc,
        callbackData: *mut c_void,
        imageAttributes: *const GpImageAttributes,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipEnumerateMetafileDestPointI"]
    pub fn GdipEnumerateMetafileDestPointI(
        graphics: *mut GpGraphics,
        metafile: *const GpMetafile,
        destPoint: *const Point,
        callback: EnumerateMetafileProc,
        callbackData: *mut c_void,
        imageAttributes: *const GpImageAttributes,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipEnumerateMetafileDestRect"]
    pub fn GdipEnumerateMetafileDestRect(
        graphics: *mut GpGraphics,
        metafile: *const GpMetafile,
        destRect: *const RectF,
        callback: EnumerateMetafileProc,
        callbackData: *mut c_void,
        imageAttributes: *const GpImageAttributes,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipEnumerateMetafileDestRectI"]
    pub fn GdipEnumerateMetafileDestRectI(
        graphics: *mut GpGraphics,
        metafile: *const GpMetafile,
        destRect: *const Rect,
        callback: EnumerateMetafileProc,
        callbackData: *mut c_void,
        imageAttributes: *const GpImageAttributes,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipEnumerateMetafileDestPoints"]
    pub fn GdipEnumerateMetafileDestPoints(
        graphics: *mut GpGraphics,
        metafile: *const GpMetafile,
        destPoints: *const PointF,
        count: INT,
        callback: EnumerateMetafileProc,
        callbackData: *mut c_void,
        imageAttributes: *const GpImageAttributes,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipEnumerateMetafileDestPointsI"]
    pub fn GdipEnumerateMetafileDestPointsI(
        graphics: *mut GpGraphics,
        metafile: *const GpMetafile,
        destPoints: *const Point,
        count: INT,
        callback: EnumerateMetafileProc,
        callbackData: *mut c_void,
        imageAttributes: *const GpImageAttributes,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipEnumerateMetafileSrcRectDestPoint"]
    pub fn GdipEnumerateMetafileSrcRectDestPoint(
        graphics: *mut GpGraphics,
        metafile: *const GpMetafile,
        destPoint: *const PointF,
        srcRect: *const RectF,
        srcUnit: Unit,
        callback: EnumerateMetafileProc,
        callbackData: *mut c_void,
        imageAttributes: *const GpImageAttributes,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipEnumerateMetafileSrcRectDestPointI"]
    pub fn GdipEnumerateMetafileSrcRectDestPointI(
        graphics: *mut GpGraphics,
        metafile: *const GpMetafile,
        destPoint: *const Point,
        srcRect: *const Rect,
        srcUnit: Unit,
        callback: EnumerateMetafileProc,
        callbackData: *mut c_void,
        imageAttributes: *const GpImageAttributes,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipEnumerateMetafileSrcRectDestRect"]
    pub fn GdipEnumerateMetafileSrcRectDestRect(
        graphics: *mut GpGraphics,
        metafile: *const GpMetafile,
        destRect: *const RectF,
        srcRect: *const RectF,
        srcUnit: Unit,
        callback: EnumerateMetafileProc,
        callbackData: *mut c_void,
        imageAttributes: *const GpImageAttributes,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipEnumerateMetafileSrcRectDestRectI"]
    pub fn GdipEnumerateMetafileSrcRectDestRectI(
        graphics: *mut GpGraphics,
        metafile: *const GpMetafile,
        destRect: *const Rect,
        srcRect: *const Rect,
        srcUnit: Unit,
        callback: EnumerateMetafileProc,
        callbackData: *mut c_void,
        imageAttributes: *const GpImageAttributes,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipEnumerateMetafileSrcRectDestPoints"]
    pub fn GdipEnumerateMetafileSrcRectDestPoints(
        graphics: *mut GpGraphics,
        metafile: *const GpMetafile,
        destPoints: *const PointF,
        count: INT,
        srcRect: *const RectF,
        srcUnit: Unit,
        callback: EnumerateMetafileProc,
        callbackData: *mut c_void,
        imageAttributes: *const GpImageAttributes,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipEnumerateMetafileSrcRectDestPointsI"]
    pub fn GdipEnumerateMetafileSrcRectDestPointsI(
        graphics: *mut GpGraphics,
        metafile: *const GpMetafile,
        destPoints: *const Point,
        count: INT,
        srcRect: *const Rect,
        srcUnit: Unit,
        callback: EnumerateMetafileProc,
        callbackData: *mut c_void,
        imageAttributes: *const GpImageAttributes,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipPlayMetafileRecord"]
    pub fn GdipPlayMetafileRecord(
        metafile: *const GpMetafile,
        recordType: EmfPlusRecordType,
        flags: UINT,
        dataSize: UINT,
        data: *const BYTE,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipSetClipGraphics"]
    pub fn GdipSetClipGraphics(
        graphics: *mut GpGraphics,
        srcgraphics: *mut GpGraphics,
        combineMode: CombineMode,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipSetClipRect"]
    pub fn GdipSetClipRect(
        graphics: *mut GpGraphics,
        x: REAL,
        y: REAL,
        width: REAL,
        height: REAL,
        combineMode: CombineMode,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipSetClipRectI"]
    pub fn GdipSetClipRectI(
        graphics: *mut GpGraphics,
        x: INT,
        y: INT,
        width: INT,
        height: INT,
        combineMode: CombineMode,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipSetClipPath"]
    pub fn GdipSetClipPath(
        graphics: *mut GpGraphics,
        path: *mut GpPath,
        combineMode: CombineMode,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipSetClipRegion"]
    pub fn GdipSetClipRegion(
        graphics: *mut GpGraphics,
        region: *mut GpRegion,
        combineMode: CombineMode,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipSetClipHrgn"]
    pub fn GdipSetClipHrgn(
        graphics: *mut GpGraphics,
        hRgn: HRGN,
        combineMode: CombineMode,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipResetClip"]
    pub fn GdipResetClip(graphics: *mut GpGraphics) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipTranslateClip"]
    pub fn GdipTranslateClip(graphics: *mut GpGraphics, dx: REAL, dy: REAL) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipTranslateClipI"]
    pub fn GdipTranslateClipI(graphics: *mut GpGraphics, dx: INT, dy: INT) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetClip"]
    pub fn GdipGetClip(graphics: *mut GpGraphics, region: *mut GpRegion) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetClipBounds"]
    pub fn GdipGetClipBounds(graphics: *mut GpGraphics, rect: *mut GpRectF) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetClipBoundsI"]
    pub fn GdipGetClipBoundsI(graphics: *mut GpGraphics, rect: *mut GpRect) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipIsClipEmpty"]
    pub fn GdipIsClipEmpty(graphics: *mut GpGraphics, result: *mut BOOL) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetVisibleClipBounds"]
    pub fn GdipGetVisibleClipBounds(graphics: *mut GpGraphics, rect: *mut GpRectF) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetVisibleClipBoundsI"]
    pub fn GdipGetVisibleClipBoundsI(graphics: *mut GpGraphics, rect: *mut GpRect) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipIsVisibleClipEmpty"]
    pub fn GdipIsVisibleClipEmpty(graphics: *mut GpGraphics, result: *mut BOOL) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipIsVisiblePoint"]
    pub fn GdipIsVisiblePoint(
        graphics: *mut GpGraphics,
        x: REAL,
        y: REAL,
        result: *mut BOOL,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipIsVisiblePointI"]
    pub fn GdipIsVisiblePointI(
        graphics: *mut GpGraphics,
        x: INT,
        y: INT,
        result: *mut BOOL,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipIsVisibleRect"]
    pub fn GdipIsVisibleRect(
        graphics: *mut GpGraphics,
        x: REAL,
        y: REAL,
        width: REAL,
        height: REAL,
        result: *mut BOOL,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipIsVisibleRectI"]
    pub fn GdipIsVisibleRectI(
        graphics: *mut GpGraphics,
        x: INT,
        y: INT,
        width: INT,
        height: INT,
        result: *mut BOOL,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipSaveGraphics"]
    pub fn GdipSaveGraphics(graphics: *mut GpGraphics, state: *mut GraphicsState) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipRestoreGraphics"]
    pub fn GdipRestoreGraphics(graphics: *mut GpGraphics, state: GraphicsState) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipBeginContainer"]
    pub fn GdipBeginContainer(
        graphics: *mut GpGraphics,
        dstrect: *const GpRectF,
        srcrect: *const GpRectF,
        unit: GpUnit,
        state: *mut GraphicsContainer,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipBeginContainerI"]
    pub fn GdipBeginContainerI(
        graphics: *mut GpGraphics,
        dstrect: *const GpRect,
        srcrect: *const GpRect,
        unit: GpUnit,
        state: *mut GraphicsContainer,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipBeginContainer2"]
    pub fn GdipBeginContainer2(
        graphics: *mut GpGraphics,
        state: *mut GraphicsContainer,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipEndContainer"]
    pub fn GdipEndContainer(graphics: *mut GpGraphics, state: GraphicsContainer) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetMetafileHeaderFromWmf"]
    pub fn GdipGetMetafileHeaderFromWmf(
        hWmf: HMETAFILE,
        wmfPlaceableFileHeader: *const WmfPlaceableFileHeader,
        header: *mut MetafileHeader,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetMetafileHeaderFromEmf"]
    pub fn GdipGetMetafileHeaderFromEmf(
        hEmf: HENHMETAFILE,
        header: *mut MetafileHeader,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetMetafileHeaderFromFile"]
    pub fn GdipGetMetafileHeaderFromFile(
        filename: *const WCHAR,
        header: *mut MetafileHeader,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetMetafileHeaderFromStream"]
    pub fn GdipGetMetafileHeaderFromStream(
        stream: *mut IStream,
        header: *mut MetafileHeader,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetMetafileHeaderFromMetafile"]
    pub fn GdipGetMetafileHeaderFromMetafile(
        metafile: *mut GpMetafile,
        header: *mut MetafileHeader,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetHemfFromMetafile"]
    pub fn GdipGetHemfFromMetafile(metafile: *mut GpMetafile, hEmf: *mut HENHMETAFILE) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipCreateStreamOnFile"]
    pub fn GdipCreateStreamOnFile(
        filename: *const WCHAR,
        access: UINT,
        stream: *mut *mut IStream,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipCreateMetafileFromWmf"]
    pub fn GdipCreateMetafileFromWmf(
        hWmf: HMETAFILE,
        deleteWmf: BOOL,
        wmfPlaceableFileHeader: *const WmfPlaceableFileHeader,
        metafile: *mut *mut GpMetafile,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipCreateMetafileFromEmf"]
    pub fn GdipCreateMetafileFromEmf(
        hEmf: HENHMETAFILE,
        deleteEmf: BOOL,
        metafile: *mut *mut GpMetafile,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipCreateMetafileFromFile"]
    pub fn GdipCreateMetafileFromFile(
        file: *const WCHAR,
        metafile: *mut *mut GpMetafile,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipCreateMetafileFromWmfFile"]
    pub fn GdipCreateMetafileFromWmfFile(
        file: *const WCHAR,
        wmfPlaceableFileHeader: *const WmfPlaceableFileHeader,
        metafile: *mut *mut GpMetafile,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipCreateMetafileFromStream"]
    pub fn GdipCreateMetafileFromStream(
        stream: *mut IStream,
        metafile: *mut *mut GpMetafile,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipRecordMetafile"]
    pub fn GdipRecordMetafile(
        referenceHdc: HDC,
        type_: EmfType,
        frameRect: *const GpRectF,
        frameUnit: MetafileFrameUnit,
        description: *const WCHAR,
        metafile: *mut *mut GpMetafile,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipRecordMetafileI"]
    pub fn GdipRecordMetafileI(
        referenceHdc: HDC,
        type_: EmfType,
        frameRect: *const GpRect,
        frameUnit: MetafileFrameUnit,
        description: *const WCHAR,
        metafile: *mut *mut GpMetafile,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipRecordMetafileFileName"]
    pub fn GdipRecordMetafileFileName(
        fileName: *const WCHAR,
        referenceHdc: HDC,
        type_: EmfType,
        frameRect: *const GpRectF,
        frameUnit: MetafileFrameUnit,
        description: *const WCHAR,
        metafile: *mut *mut GpMetafile,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipRecordMetafileFileNameI"]
    pub fn GdipRecordMetafileFileNameI(
        fileName: *const WCHAR,
        referenceHdc: HDC,
        type_: EmfType,
        frameRect: *const GpRect,
        frameUnit: MetafileFrameUnit,
        description: *const WCHAR,
        metafile: *mut *mut GpMetafile,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipRecordMetafileStream"]
    pub fn GdipRecordMetafileStream(
        stream: *mut IStream,
        referenceHdc: HDC,
        type_: EmfType,
        frameRect: *const GpRectF,
        frameUnit: MetafileFrameUnit,
        description: *const WCHAR,
        metafile: *mut *mut GpMetafile,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipRecordMetafileStreamI"]
    pub fn GdipRecordMetafileStreamI(
        stream: *mut IStream,
        referenceHdc: HDC,
        type_: EmfType,
        frameRect: *const GpRect,
        frameUnit: MetafileFrameUnit,
        description: *const WCHAR,
        metafile: *mut *mut GpMetafile,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipSetMetafileDownLevelRasterizationLimit"]
    pub fn GdipSetMetafileDownLevelRasterizationLimit(
        metafile: *mut GpMetafile,
        metafileRasterizationLimitDpi: UINT,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetMetafileDownLevelRasterizationLimit"]
    pub fn GdipGetMetafileDownLevelRasterizationLimit(
        metafile: *const GpMetafile,
        metafileRasterizationLimitDpi: *mut UINT,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetImageDecodersSize"]
    pub fn GdipGetImageDecodersSize(numDecoders: *mut UINT, size: *mut UINT) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetImageDecoders"]
    pub fn GdipGetImageDecoders(
        numDecoders: UINT,
        size: UINT,
        decoders: *mut ImageCodecInfo,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetImageEncodersSize"]
    pub fn GdipGetImageEncodersSize(numEncoders: *mut UINT, size: *mut UINT) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetImageEncoders"]
    pub fn GdipGetImageEncoders(
        numEncoders: UINT,
        size: UINT,
        encoders: *mut ImageCodecInfo,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipComment"]
    pub fn GdipComment(graphics: *mut GpGraphics, sizeData: UINT, data: *const BYTE) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipCreateFontFamilyFromName"]
    pub fn GdipCreateFontFamilyFromName(
        name: *const WCHAR,
        fontCollection: *mut GpFontCollection,
        fontFamily: *mut *mut GpFontFamily,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipDeleteFontFamily"]
    pub fn GdipDeleteFontFamily(fontFamily: *mut GpFontFamily) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipCloneFontFamily"]
    pub fn GdipCloneFontFamily(
        fontFamily: *mut GpFontFamily,
        clonedFontFamily: *mut *mut GpFontFamily,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetGenericFontFamilySansSerif"]
    pub fn GdipGetGenericFontFamilySansSerif(nativeFamily: *mut *mut GpFontFamily) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetGenericFontFamilySerif"]
    pub fn GdipGetGenericFontFamilySerif(nativeFamily: *mut *mut GpFontFamily) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetGenericFontFamilyMonospace"]
    pub fn GdipGetGenericFontFamilyMonospace(nativeFamily: *mut *mut GpFontFamily) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetFamilyName"]
    pub fn GdipGetFamilyName(
        family: *const GpFontFamily,
        name: LPWSTR,
        language: LANGID,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipIsStyleAvailable"]
    pub fn GdipIsStyleAvailable(
        family: *const GpFontFamily,
        style: INT,
        IsStyleAvailable: *mut BOOL,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipFontCollectionEnumerable"]
    pub fn GdipFontCollectionEnumerable(
        fontCollection: *mut GpFontCollection,
        graphics: *mut GpGraphics,
        numFound: *mut INT,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipFontCollectionEnumerate"]
    pub fn GdipFontCollectionEnumerate(
        fontCollection: *mut GpFontCollection,
        numSought: INT,
        gpfamilies: *mut *mut GpFontFamily,
        numFound: *mut INT,
        graphics: *mut GpGraphics,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetEmHeight"]
    pub fn GdipGetEmHeight(
        family: *const GpFontFamily,
        style: INT,
        EmHeight: *mut UINT16,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetCellAscent"]
    pub fn GdipGetCellAscent(
        family: *const GpFontFamily,
        style: INT,
        CellAscent: *mut UINT16,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetCellDescent"]
    pub fn GdipGetCellDescent(
        family: *const GpFontFamily,
        style: INT,
        CellDescent: *mut UINT16,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetLineSpacing"]
    pub fn GdipGetLineSpacing(
        family: *const GpFontFamily,
        style: INT,
        LineSpacing: *mut UINT16,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipCreateFontFromDC"]
    pub fn GdipCreateFontFromDC(hdc: HDC, font: *mut *mut GpFont) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipCreateFontFromLogfontA"]
    pub fn GdipCreateFontFromLogfontA(
        hdc: HDC,
        logfont: *const LOGFONTA,
        font: *mut *mut GpFont,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipCreateFontFromLogfontW"]
    pub fn GdipCreateFontFromLogfontW(
        hdc: HDC,
        logfont: *const LOGFONTW,
        font: *mut *mut GpFont,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipCreateFont"]
    pub fn GdipCreateFont(
        fontFamily: *const GpFontFamily,
        emSize: REAL,
        style: INT,
        unit: Unit,
        font: *mut *mut GpFont,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipCloneFont"]
    pub fn GdipCloneFont(font: *mut GpFont, cloneFont: *mut *mut GpFont) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipDeleteFont"]
    pub fn GdipDeleteFont(font: *mut GpFont) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetFamily"]
    pub fn GdipGetFamily(font: *mut GpFont, family: *mut *mut GpFontFamily) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetFontStyle"]
    pub fn GdipGetFontStyle(font: *mut GpFont, style: *mut INT) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetFontSize"]
    pub fn GdipGetFontSize(font: *mut GpFont, size: *mut REAL) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetFontUnit"]
    pub fn GdipGetFontUnit(font: *mut GpFont, unit: *mut Unit) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetFontHeight"]
    pub fn GdipGetFontHeight(
        font: *const GpFont,
        graphics: *const GpGraphics,
        height: *mut REAL,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetFontHeightGivenDPI"]
    pub fn GdipGetFontHeightGivenDPI(font: *const GpFont, dpi: REAL, height: *mut REAL)
        -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetLogFontA"]
    pub fn GdipGetLogFontA(
        font: *mut GpFont,
        graphics: *mut GpGraphics,
        logfontA: *mut LOGFONTA,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetLogFontW"]
    pub fn GdipGetLogFontW(
        font: *mut GpFont,
        graphics: *mut GpGraphics,
        logfontW: *mut LOGFONTW,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipNewInstalledFontCollection"]
    pub fn GdipNewInstalledFontCollection(fontCollection: *mut *mut GpFontCollection) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipNewPrivateFontCollection"]
    pub fn GdipNewPrivateFontCollection(fontCollection: *mut *mut GpFontCollection) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipDeletePrivateFontCollection"]
    pub fn GdipDeletePrivateFontCollection(fontCollection: *mut *mut GpFontCollection) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetFontCollectionFamilyCount"]
    pub fn GdipGetFontCollectionFamilyCount(
        fontCollection: *mut GpFontCollection,
        numFound: *mut INT,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetFontCollectionFamilyList"]
    pub fn GdipGetFontCollectionFamilyList(
        fontCollection: *mut GpFontCollection,
        numSought: INT,
        gpfamilies: *mut *mut GpFontFamily,
        numFound: *mut INT,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipPrivateAddFontFile"]
    pub fn GdipPrivateAddFontFile(
        fontCollection: *mut GpFontCollection,
        filename: *const WCHAR,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipPrivateAddMemoryFont"]
    pub fn GdipPrivateAddMemoryFont(
        fontCollection: *mut GpFontCollection,
        memory: *const c_void,
        length: INT,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipDrawString"]
    pub fn GdipDrawString(
        graphics: *mut GpGraphics,
        string: *const WCHAR,
        length: INT,
        font: *const GpFont,
        layoutRect: *const RectF,
        stringFormat: *const GpStringFormat,
        brush: *const GpBrush,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipMeasureString"]
    pub fn GdipMeasureString(
        graphics: *mut GpGraphics,
        string: *const WCHAR,
        length: INT,
        font: *const GpFont,
        layoutRect: *const RectF,
        stringFormat: *const GpStringFormat,
        boundingBox: *mut RectF,
        codepointsFitted: *mut INT,
        linesFilled: *mut INT,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipMeasureCharacterRanges"]
    pub fn GdipMeasureCharacterRanges(
        graphics: *mut GpGraphics,
        string: *const WCHAR,
        length: INT,
        font: *const GpFont,
        layoutRect: *const RectF,
        stringFormat: *const GpStringFormat,
        regionCount: INT,
        regions: *mut *mut GpRegion,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipDrawDriverString"]
    pub fn GdipDrawDriverString(
        graphics: *mut GpGraphics,
        text: *const UINT16,
        length: INT,
        font: *const GpFont,
        brush: *const GpBrush,
        positions: *const PointF,
        flags: INT,
        matrix: *const GpMatrix,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipMeasureDriverString"]
    pub fn GdipMeasureDriverString(
        graphics: *mut GpGraphics,
        text: *const UINT16,
        length: INT,
        font: *const GpFont,
        positions: *const PointF,
        flags: INT,
        matrix: *const GpMatrix,
        boundingBox: *mut RectF,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipCreateStringFormat"]
    pub fn GdipCreateStringFormat(
        formatAttributes: INT,
        language: LANGID,
        format: *mut *mut GpStringFormat,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipStringFormatGetGenericDefault"]
    pub fn GdipStringFormatGetGenericDefault(format: *mut *mut GpStringFormat) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipStringFormatGetGenericTypographic"]
    pub fn GdipStringFormatGetGenericTypographic(format: *mut *mut GpStringFormat) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipDeleteStringFormat"]
    pub fn GdipDeleteStringFormat(format: *mut GpStringFormat) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipCloneStringFormat"]
    pub fn GdipCloneStringFormat(
        format: *const GpStringFormat,
        newFormat: *mut *mut GpStringFormat,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipSetStringFormatFlags"]
    pub fn GdipSetStringFormatFlags(format: *mut GpStringFormat, flags: INT) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetStringFormatFlags"]
    pub fn GdipGetStringFormatFlags(format: *const GpStringFormat, flags: *mut INT) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipSetStringFormatAlign"]
    pub fn GdipSetStringFormatAlign(
        format: *mut GpStringFormat,
        align: StringAlignment,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetStringFormatAlign"]
    pub fn GdipGetStringFormatAlign(
        format: *const GpStringFormat,
        align: *mut StringAlignment,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipSetStringFormatLineAlign"]
    pub fn GdipSetStringFormatLineAlign(
        format: *mut GpStringFormat,
        align: StringAlignment,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetStringFormatLineAlign"]
    pub fn GdipGetStringFormatLineAlign(
        format: *const GpStringFormat,
        align: *mut StringAlignment,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipSetStringFormatTrimming"]
    pub fn GdipSetStringFormatTrimming(
        format: *mut GpStringFormat,
        trimming: StringTrimming,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetStringFormatTrimming"]
    pub fn GdipGetStringFormatTrimming(
        format: *const GpStringFormat,
        trimming: *mut StringTrimming,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipSetStringFormatHotkeyPrefix"]
    pub fn GdipSetStringFormatHotkeyPrefix(
        format: *mut GpStringFormat,
        hotkeyPrefix: INT,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetStringFormatHotkeyPrefix"]
    pub fn GdipGetStringFormatHotkeyPrefix(
        format: *const GpStringFormat,
        hotkeyPrefix: *mut INT,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipSetStringFormatTabStops"]
    pub fn GdipSetStringFormatTabStops(
        format: *mut GpStringFormat,
        firstTabOffset: REAL,
        count: INT,
        tabStops: *const REAL,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetStringFormatTabStops"]
    pub fn GdipGetStringFormatTabStops(
        format: *const GpStringFormat,
        count: INT,
        firstTabOffset: *mut REAL,
        tabStops: *mut REAL,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetStringFormatTabStopCount"]
    pub fn GdipGetStringFormatTabStopCount(
        format: *const GpStringFormat,
        count: *mut INT,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipSetStringFormatDigitSubstitution"]
    pub fn GdipSetStringFormatDigitSubstitution(
        format: *mut GpStringFormat,
        language: LANGID,
        substitute: StringDigitSubstitute,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetStringFormatDigitSubstitution"]
    pub fn GdipGetStringFormatDigitSubstitution(
        format: *const GpStringFormat,
        language: *mut LANGID,
        substitute: *mut StringDigitSubstitute,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipGetStringFormatMeasurableCharacterRangeCount"]
    pub fn GdipGetStringFormatMeasurableCharacterRangeCount(
        format: *const GpStringFormat,
        count: *mut INT,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipSetStringFormatMeasurableCharacterRanges"]
    pub fn GdipSetStringFormatMeasurableCharacterRanges(
        format: *mut GpStringFormat,
        rangeCount: INT,
        ranges: *const CharacterRange,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipCreateCachedBitmap"]
    pub fn GdipCreateCachedBitmap(
        bitmap: *mut GpBitmap,
        graphics: *mut GpGraphics,
        cachedBitmap: *mut *mut GpCachedBitmap,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipDeleteCachedBitmap"]
    pub fn GdipDeleteCachedBitmap(cachedBitmap: *mut GpCachedBitmap) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipDrawCachedBitmap"]
    pub fn GdipDrawCachedBitmap(
        graphics: *mut GpGraphics,
        cachedBitmap: *mut GpCachedBitmap,
        x: INT,
        y: INT,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipEmfToWmfBits"]
    pub fn GdipEmfToWmfBits(
        hemf: HENHMETAFILE,
        cbData16: UINT,
        pData16: LPBYTE,
        iMapMode: INT,
        eFlags: INT,
    ) -> UINT;
}
extern "C" {
    #[link_name = "\u{1}GdipSetImageAttributesCachedBackground"]
    pub fn GdipSetImageAttributesCachedBackground(
        imageattr: *mut GpImageAttributes,
        enableFlag: BOOL,
    ) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdipTestControl"]
    pub fn GdipTestControl(control: GpTestControlEnum, param: *mut c_void) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdiplusNotificationHook"]
    pub fn GdiplusNotificationHook(token: *mut ULONG_PTR) -> GpStatus;
}
extern "C" {
    #[link_name = "\u{1}GdiplusNotificationUnhook"]
    pub fn GdiplusNotificationUnhook(token: ULONG_PTR);
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct GdiplusBase {
    pub _address: u8,
}
pub type GraphicsState = UINT;
pub type GraphicsContainer = UINT;
pub const FillMode_FillModeAlternate: FillMode = 0;
pub const FillMode_FillModeWinding: FillMode = 1;
pub type FillMode = c_int;
pub const CompositingMode_CompositingModeSourceOver: CompositingMode = 0;
pub const CompositingMode_CompositingModeSourceCopy: CompositingMode = 1;
pub type CompositingMode = c_int;
pub const CompositingQuality_CompositingQualityInvalid: CompositingQuality = -1;
pub const CompositingQuality_CompositingQualityDefault: CompositingQuality = 0;
pub const CompositingQuality_CompositingQualityHighSpeed: CompositingQuality = 1;
pub const CompositingQuality_CompositingQualityHighQuality: CompositingQuality = 2;
pub const CompositingQuality_CompositingQualityGammaCorrected: CompositingQuality = 3;
pub const CompositingQuality_CompositingQualityAssumeLinear: CompositingQuality = 4;
pub type CompositingQuality = c_int;
pub const Unit_UnitWorld: Unit = 0;
pub const Unit_UnitDisplay: Unit = 1;
pub const Unit_UnitPixel: Unit = 2;
pub const Unit_UnitPoint: Unit = 3;
pub const Unit_UnitInch: Unit = 4;
pub const Unit_UnitDocument: Unit = 5;
pub const Unit_UnitMillimeter: Unit = 6;
pub type Unit = c_int;
pub const MetafileFrameUnit_MetafileFrameUnitPixel: MetafileFrameUnit = 2;
pub const MetafileFrameUnit_MetafileFrameUnitPoint: MetafileFrameUnit = 3;
pub const MetafileFrameUnit_MetafileFrameUnitInch: MetafileFrameUnit = 4;
pub const MetafileFrameUnit_MetafileFrameUnitDocument: MetafileFrameUnit = 5;
pub const MetafileFrameUnit_MetafileFrameUnitMillimeter: MetafileFrameUnit = 6;
pub const MetafileFrameUnit_MetafileFrameUnitGdi: MetafileFrameUnit = 7;
pub type MetafileFrameUnit = c_int;
pub const CoordinateSpace_CoordinateSpaceWorld: CoordinateSpace = 0;
pub const CoordinateSpace_CoordinateSpacePage: CoordinateSpace = 1;
pub const CoordinateSpace_CoordinateSpaceDevice: CoordinateSpace = 2;
pub type CoordinateSpace = c_int;
pub const WrapMode_WrapModeTile: WrapMode = 0;
pub const WrapMode_WrapModeTileFlipX: WrapMode = 1;
pub const WrapMode_WrapModeTileFlipY: WrapMode = 2;
pub const WrapMode_WrapModeTileFlipXY: WrapMode = 3;
pub const WrapMode_WrapModeClamp: WrapMode = 4;
pub type WrapMode = c_int;
pub const HatchStyle_HatchStyleHorizontal: HatchStyle = 0;
pub const HatchStyle_HatchStyleVertical: HatchStyle = 1;
pub const HatchStyle_HatchStyleForwardDiagonal: HatchStyle = 2;
pub const HatchStyle_HatchStyleBackwardDiagonal: HatchStyle = 3;
pub const HatchStyle_HatchStyleCross: HatchStyle = 4;
pub const HatchStyle_HatchStyleDiagonalCross: HatchStyle = 5;
pub const HatchStyle_HatchStyle05Percent: HatchStyle = 6;
pub const HatchStyle_HatchStyle10Percent: HatchStyle = 7;
pub const HatchStyle_HatchStyle20Percent: HatchStyle = 8;
pub const HatchStyle_HatchStyle25Percent: HatchStyle = 9;
pub const HatchStyle_HatchStyle30Percent: HatchStyle = 10;
pub const HatchStyle_HatchStyle40Percent: HatchStyle = 11;
pub const HatchStyle_HatchStyle50Percent: HatchStyle = 12;
pub const HatchStyle_HatchStyle60Percent: HatchStyle = 13;
pub const HatchStyle_HatchStyle70Percent: HatchStyle = 14;
pub const HatchStyle_HatchStyle75Percent: HatchStyle = 15;
pub const HatchStyle_HatchStyle80Percent: HatchStyle = 16;
pub const HatchStyle_HatchStyle90Percent: HatchStyle = 17;
pub const HatchStyle_HatchStyleLightDownwardDiagonal: HatchStyle = 18;
pub const HatchStyle_HatchStyleLightUpwardDiagonal: HatchStyle = 19;
pub const HatchStyle_HatchStyleDarkDownwardDiagonal: HatchStyle = 20;
pub const HatchStyle_HatchStyleDarkUpwardDiagonal: HatchStyle = 21;
pub const HatchStyle_HatchStyleWideDownwardDiagonal: HatchStyle = 22;
pub const HatchStyle_HatchStyleWideUpwardDiagonal: HatchStyle = 23;
pub const HatchStyle_HatchStyleLightVertical: HatchStyle = 24;
pub const HatchStyle_HatchStyleLightHorizontal: HatchStyle = 25;
pub const HatchStyle_HatchStyleNarrowVertical: HatchStyle = 26;
pub const HatchStyle_HatchStyleNarrowHorizontal: HatchStyle = 27;
pub const HatchStyle_HatchStyleDarkVertical: HatchStyle = 28;
pub const HatchStyle_HatchStyleDarkHorizontal: HatchStyle = 29;
pub const HatchStyle_HatchStyleDashedDownwardDiagonal: HatchStyle = 30;
pub const HatchStyle_HatchStyleDashedUpwardDiagonal: HatchStyle = 31;
pub const HatchStyle_HatchStyleDashedHorizontal: HatchStyle = 32;
pub const HatchStyle_HatchStyleDashedVertical: HatchStyle = 33;
pub const HatchStyle_HatchStyleSmallConfetti: HatchStyle = 34;
pub const HatchStyle_HatchStyleLargeConfetti: HatchStyle = 35;
pub const HatchStyle_HatchStyleZigZag: HatchStyle = 36;
pub const HatchStyle_HatchStyleWave: HatchStyle = 37;
pub const HatchStyle_HatchStyleDiagonalBrick: HatchStyle = 38;
pub const HatchStyle_HatchStyleHorizontalBrick: HatchStyle = 39;
pub const HatchStyle_HatchStyleWeave: HatchStyle = 40;
pub const HatchStyle_HatchStylePlaid: HatchStyle = 41;
pub const HatchStyle_HatchStyleDivot: HatchStyle = 42;
pub const HatchStyle_HatchStyleDottedGrid: HatchStyle = 43;
pub const HatchStyle_HatchStyleDottedDiamond: HatchStyle = 44;
pub const HatchStyle_HatchStyleShingle: HatchStyle = 45;
pub const HatchStyle_HatchStyleTrellis: HatchStyle = 46;
pub const HatchStyle_HatchStyleSphere: HatchStyle = 47;
pub const HatchStyle_HatchStyleSmallGrid: HatchStyle = 48;
pub const HatchStyle_HatchStyleSmallCheckerBoard: HatchStyle = 49;
pub const HatchStyle_HatchStyleLargeCheckerBoard: HatchStyle = 50;
pub const HatchStyle_HatchStyleOutlinedDiamond: HatchStyle = 51;
pub const HatchStyle_HatchStyleSolidDiamond: HatchStyle = 52;
pub const HatchStyle_HatchStyleTotal: HatchStyle = 53;
pub const HatchStyle_HatchStyleLargeGrid: HatchStyle = 4;
pub const HatchStyle_HatchStyleMin: HatchStyle = 0;
pub const HatchStyle_HatchStyleMax: HatchStyle = 52;
pub type HatchStyle = c_int;
pub const DashStyle_DashStyleSolid: DashStyle = 0;
pub const DashStyle_DashStyleDash: DashStyle = 1;
pub const DashStyle_DashStyleDot: DashStyle = 2;
pub const DashStyle_DashStyleDashDot: DashStyle = 3;
pub const DashStyle_DashStyleDashDotDot: DashStyle = 4;
pub const DashStyle_DashStyleCustom: DashStyle = 5;
pub type DashStyle = c_int;
pub const DashCap_DashCapFlat: DashCap = 0;
pub const DashCap_DashCapRound: DashCap = 2;
pub const DashCap_DashCapTriangle: DashCap = 3;
pub type DashCap = c_int;
pub const LineCap_LineCapFlat: LineCap = 0;
pub const LineCap_LineCapSquare: LineCap = 1;
pub const LineCap_LineCapRound: LineCap = 2;
pub const LineCap_LineCapTriangle: LineCap = 3;
pub const LineCap_LineCapNoAnchor: LineCap = 16;
pub const LineCap_LineCapSquareAnchor: LineCap = 17;
pub const LineCap_LineCapRoundAnchor: LineCap = 18;
pub const LineCap_LineCapDiamondAnchor: LineCap = 19;
pub const LineCap_LineCapArrowAnchor: LineCap = 20;
pub const LineCap_LineCapCustom: LineCap = 255;
pub const LineCap_LineCapAnchorMask: LineCap = 240;
pub type LineCap = c_int;
pub const CustomLineCapType_CustomLineCapTypeDefault: CustomLineCapType = 0;
pub const CustomLineCapType_CustomLineCapTypeAdjustableArrow: CustomLineCapType = 1;
pub type CustomLineCapType = c_int;
pub const LineJoin_LineJoinMiter: LineJoin = 0;
pub const LineJoin_LineJoinBevel: LineJoin = 1;
pub const LineJoin_LineJoinRound: LineJoin = 2;
pub const LineJoin_LineJoinMiterClipped: LineJoin = 3;
pub type LineJoin = c_int;
pub const WarpMode_WarpModePerspective: WarpMode = 0;
pub const WarpMode_WarpModeBilinear: WarpMode = 1;
pub type WarpMode = c_int;
pub const LinearGradientMode_LinearGradientModeHorizontal: LinearGradientMode = 0;
pub const LinearGradientMode_LinearGradientModeVertical: LinearGradientMode = 1;
pub const LinearGradientMode_LinearGradientModeForwardDiagonal: LinearGradientMode = 2;
pub const LinearGradientMode_LinearGradientModeBackwardDiagonal: LinearGradientMode = 3;
pub type LinearGradientMode = c_int;
pub const CombineMode_CombineModeReplace: CombineMode = 0;
pub const CombineMode_CombineModeIntersect: CombineMode = 1;
pub const CombineMode_CombineModeUnion: CombineMode = 2;
pub const CombineMode_CombineModeXor: CombineMode = 3;
pub const CombineMode_CombineModeExclude: CombineMode = 4;
pub const CombineMode_CombineModeComplement: CombineMode = 5;
pub type CombineMode = c_int;
pub const ImageType_ImageTypeUnknown: ImageType = 0;
pub const ImageType_ImageTypeBitmap: ImageType = 1;
pub const ImageType_ImageTypeMetafile: ImageType = 2;
pub type ImageType = c_int;
pub const InterpolationMode_InterpolationModeInvalid: InterpolationMode = -1;
pub const InterpolationMode_InterpolationModeDefault: InterpolationMode = 0;
pub const InterpolationMode_InterpolationModeLowQuality: InterpolationMode = 1;
pub const InterpolationMode_InterpolationModeHighQuality: InterpolationMode = 2;
pub const InterpolationMode_InterpolationModeBilinear: InterpolationMode = 3;
pub const InterpolationMode_InterpolationModeBicubic: InterpolationMode = 4;
pub const InterpolationMode_InterpolationModeNearestNeighbor: InterpolationMode = 5;
pub const InterpolationMode_InterpolationModeHighQualityBilinear: InterpolationMode = 6;
pub const InterpolationMode_InterpolationModeHighQualityBicubic: InterpolationMode = 7;
pub type InterpolationMode = c_int;
pub const PenAlignment_PenAlignmentCenter: PenAlignment = 0;
pub const PenAlignment_PenAlignmentInset: PenAlignment = 1;
pub type PenAlignment = c_int;
pub const BrushType_BrushTypeSolidColor: BrushType = 0;
pub const BrushType_BrushTypeHatchFill: BrushType = 1;
pub const BrushType_BrushTypeTextureFill: BrushType = 2;
pub const BrushType_BrushTypePathGradient: BrushType = 3;
pub const BrushType_BrushTypeLinearGradient: BrushType = 4;
pub type BrushType = c_int;
pub const PenType_PenTypeSolidColor: PenType = 0;
pub const PenType_PenTypeHatchFill: PenType = 1;
pub const PenType_PenTypeTextureFill: PenType = 2;
pub const PenType_PenTypePathGradient: PenType = 3;
pub const PenType_PenTypeLinearGradient: PenType = 4;
pub const PenType_PenTypeUnknown: PenType = -1;
pub type PenType = c_int;
pub const MatrixOrder_MatrixOrderPrepend: MatrixOrder = 0;
pub const MatrixOrder_MatrixOrderAppend: MatrixOrder = 1;
pub type MatrixOrder = c_int;
pub const SmoothingMode_SmoothingModeInvalid: SmoothingMode = -1;
pub const SmoothingMode_SmoothingModeDefault: SmoothingMode = 0;
pub const SmoothingMode_SmoothingModeHighSpeed: SmoothingMode = 1;
pub const SmoothingMode_SmoothingModeHighQuality: SmoothingMode = 2;
pub const SmoothingMode_SmoothingModeNone: SmoothingMode = 3;
pub const SmoothingMode_SmoothingModeAntiAlias: SmoothingMode = 4;
pub type SmoothingMode = c_int;
pub const PixelOffsetMode_PixelOffsetModeInvalid: PixelOffsetMode = -1;
pub const PixelOffsetMode_PixelOffsetModeDefault: PixelOffsetMode = 0;
pub const PixelOffsetMode_PixelOffsetModeHighSpeed: PixelOffsetMode = 1;
pub const PixelOffsetMode_PixelOffsetModeHighQuality: PixelOffsetMode = 2;
pub const PixelOffsetMode_PixelOffsetModeNone: PixelOffsetMode = 3;
pub const PixelOffsetMode_PixelOffsetModeHalf: PixelOffsetMode = 4;
pub type PixelOffsetMode = c_int;
pub const TextRenderingHint_TextRenderingHintSystemDefault: TextRenderingHint = 0;
pub const TextRenderingHint_TextRenderingHintSingleBitPerPixelGridFit: TextRenderingHint = 1;
pub const TextRenderingHint_TextRenderingHintSingleBitPerPixel: TextRenderingHint = 2;
pub const TextRenderingHint_TextRenderingHintAntiAliasGridFit: TextRenderingHint = 3;
pub const TextRenderingHint_TextRenderingHintAntiAlias: TextRenderingHint = 4;
pub const TextRenderingHint_TextRenderingHintClearTypeGridFit: TextRenderingHint = 5;
pub type TextRenderingHint = c_int;
pub const MetafileType_MetafileTypeInvalid: MetafileType = 0;
pub const MetafileType_MetafileTypeWmf: MetafileType = 1;
pub const MetafileType_MetafileTypeWmfPlaceable: MetafileType = 2;
pub const MetafileType_MetafileTypeEmf: MetafileType = 3;
pub const MetafileType_MetafileTypeEmfPlusOnly: MetafileType = 4;
pub const MetafileType_MetafileTypeEmfPlusDual: MetafileType = 5;
pub type MetafileType = c_int;
pub const EmfType_EmfTypeEmfOnly: EmfType = 3;
pub const EmfType_EmfTypeEmfPlusOnly: EmfType = 4;
pub const EmfType_EmfTypeEmfPlusDual: EmfType = 5;
pub type EmfType = c_int;
pub const EmfPlusRecordType_WmfRecordTypeSetBkColor: EmfPlusRecordType = 66049;
pub const EmfPlusRecordType_WmfRecordTypeSetBkMode: EmfPlusRecordType = 65794;
pub const EmfPlusRecordType_WmfRecordTypeSetMapMode: EmfPlusRecordType = 65795;
pub const EmfPlusRecordType_WmfRecordTypeSetROP2: EmfPlusRecordType = 65796;
pub const EmfPlusRecordType_WmfRecordTypeSetRelAbs: EmfPlusRecordType = 65797;
pub const EmfPlusRecordType_WmfRecordTypeSetPolyFillMode: EmfPlusRecordType = 65798;
pub const EmfPlusRecordType_WmfRecordTypeSetStretchBltMode: EmfPlusRecordType = 65799;
pub const EmfPlusRecordType_WmfRecordTypeSetTextCharExtra: EmfPlusRecordType = 65800;
pub const EmfPlusRecordType_WmfRecordTypeSetTextColor: EmfPlusRecordType = 66057;
pub const EmfPlusRecordType_WmfRecordTypeSetTextJustification: EmfPlusRecordType = 66058;
pub const EmfPlusRecordType_WmfRecordTypeSetWindowOrg: EmfPlusRecordType = 66059;
pub const EmfPlusRecordType_WmfRecordTypeSetWindowExt: EmfPlusRecordType = 66060;
pub const EmfPlusRecordType_WmfRecordTypeSetViewportOrg: EmfPlusRecordType = 66061;
pub const EmfPlusRecordType_WmfRecordTypeSetViewportExt: EmfPlusRecordType = 66062;
pub const EmfPlusRecordType_WmfRecordTypeOffsetWindowOrg: EmfPlusRecordType = 66063;
pub const EmfPlusRecordType_WmfRecordTypeScaleWindowExt: EmfPlusRecordType = 66576;
pub const EmfPlusRecordType_WmfRecordTypeOffsetViewportOrg: EmfPlusRecordType = 66065;
pub const EmfPlusRecordType_WmfRecordTypeScaleViewportExt: EmfPlusRecordType = 66578;
pub const EmfPlusRecordType_WmfRecordTypeLineTo: EmfPlusRecordType = 66067;
pub const EmfPlusRecordType_WmfRecordTypeMoveTo: EmfPlusRecordType = 66068;
pub const EmfPlusRecordType_WmfRecordTypeExcludeClipRect: EmfPlusRecordType = 66581;
pub const EmfPlusRecordType_WmfRecordTypeIntersectClipRect: EmfPlusRecordType = 66582;
pub const EmfPlusRecordType_WmfRecordTypeArc: EmfPlusRecordType = 67607;
pub const EmfPlusRecordType_WmfRecordTypeEllipse: EmfPlusRecordType = 66584;
pub const EmfPlusRecordType_WmfRecordTypeFloodFill: EmfPlusRecordType = 66585;
pub const EmfPlusRecordType_WmfRecordTypePie: EmfPlusRecordType = 67610;
pub const EmfPlusRecordType_WmfRecordTypeRectangle: EmfPlusRecordType = 66587;
pub const EmfPlusRecordType_WmfRecordTypeRoundRect: EmfPlusRecordType = 67100;
pub const EmfPlusRecordType_WmfRecordTypePatBlt: EmfPlusRecordType = 67101;
pub const EmfPlusRecordType_WmfRecordTypeSaveDC: EmfPlusRecordType = 65566;
pub const EmfPlusRecordType_WmfRecordTypeSetPixel: EmfPlusRecordType = 66591;
pub const EmfPlusRecordType_WmfRecordTypeOffsetClipRgn: EmfPlusRecordType = 66080;
pub const EmfPlusRecordType_WmfRecordTypeTextOut: EmfPlusRecordType = 66849;
pub const EmfPlusRecordType_WmfRecordTypeBitBlt: EmfPlusRecordType = 67874;
pub const EmfPlusRecordType_WmfRecordTypeStretchBlt: EmfPlusRecordType = 68387;
pub const EmfPlusRecordType_WmfRecordTypePolygon: EmfPlusRecordType = 66340;
pub const EmfPlusRecordType_WmfRecordTypePolyline: EmfPlusRecordType = 66341;
pub const EmfPlusRecordType_WmfRecordTypeEscape: EmfPlusRecordType = 67110;
pub const EmfPlusRecordType_WmfRecordTypeRestoreDC: EmfPlusRecordType = 65831;
pub const EmfPlusRecordType_WmfRecordTypeFillRegion: EmfPlusRecordType = 66088;
pub const EmfPlusRecordType_WmfRecordTypeFrameRegion: EmfPlusRecordType = 66601;
pub const EmfPlusRecordType_WmfRecordTypeInvertRegion: EmfPlusRecordType = 65834;
pub const EmfPlusRecordType_WmfRecordTypePaintRegion: EmfPlusRecordType = 65835;
pub const EmfPlusRecordType_WmfRecordTypeSelectClipRegion: EmfPlusRecordType = 65836;
pub const EmfPlusRecordType_WmfRecordTypeSelectObject: EmfPlusRecordType = 65837;
pub const EmfPlusRecordType_WmfRecordTypeSetTextAlign: EmfPlusRecordType = 65838;
pub const EmfPlusRecordType_WmfRecordTypeDrawText: EmfPlusRecordType = 67119;
pub const EmfPlusRecordType_WmfRecordTypeChord: EmfPlusRecordType = 67632;
pub const EmfPlusRecordType_WmfRecordTypeSetMapperFlags: EmfPlusRecordType = 66097;
pub const EmfPlusRecordType_WmfRecordTypeExtTextOut: EmfPlusRecordType = 68146;
pub const EmfPlusRecordType_WmfRecordTypeSetDIBToDev: EmfPlusRecordType = 68915;
pub const EmfPlusRecordType_WmfRecordTypeSelectPalette: EmfPlusRecordType = 66100;
pub const EmfPlusRecordType_WmfRecordTypeRealizePalette: EmfPlusRecordType = 65589;
pub const EmfPlusRecordType_WmfRecordTypeAnimatePalette: EmfPlusRecordType = 66614;
pub const EmfPlusRecordType_WmfRecordTypeSetPalEntries: EmfPlusRecordType = 65591;
pub const EmfPlusRecordType_WmfRecordTypePolyPolygon: EmfPlusRecordType = 66872;
pub const EmfPlusRecordType_WmfRecordTypeResizePalette: EmfPlusRecordType = 65849;
pub const EmfPlusRecordType_WmfRecordTypeDIBBitBlt: EmfPlusRecordType = 67904;
pub const EmfPlusRecordType_WmfRecordTypeDIBStretchBlt: EmfPlusRecordType = 68417;
pub const EmfPlusRecordType_WmfRecordTypeDIBCreatePatternBrush: EmfPlusRecordType = 65858;
pub const EmfPlusRecordType_WmfRecordTypeStretchDIB: EmfPlusRecordType = 69443;
pub const EmfPlusRecordType_WmfRecordTypeExtFloodFill: EmfPlusRecordType = 66888;
pub const EmfPlusRecordType_WmfRecordTypeSetLayout: EmfPlusRecordType = 65865;
pub const EmfPlusRecordType_WmfRecordTypeResetDC: EmfPlusRecordType = 65868;
pub const EmfPlusRecordType_WmfRecordTypeStartDoc: EmfPlusRecordType = 65869;
pub const EmfPlusRecordType_WmfRecordTypeStartPage: EmfPlusRecordType = 65615;
pub const EmfPlusRecordType_WmfRecordTypeEndPage: EmfPlusRecordType = 65616;
pub const EmfPlusRecordType_WmfRecordTypeAbortDoc: EmfPlusRecordType = 65618;
pub const EmfPlusRecordType_WmfRecordTypeEndDoc: EmfPlusRecordType = 65630;
pub const EmfPlusRecordType_WmfRecordTypeDeleteObject: EmfPlusRecordType = 66032;
pub const EmfPlusRecordType_WmfRecordTypeCreatePalette: EmfPlusRecordType = 65783;
pub const EmfPlusRecordType_WmfRecordTypeCreateBrush: EmfPlusRecordType = 65784;
pub const EmfPlusRecordType_WmfRecordTypeCreatePatternBrush: EmfPlusRecordType = 66041;
pub const EmfPlusRecordType_WmfRecordTypeCreatePenIndirect: EmfPlusRecordType = 66298;
pub const EmfPlusRecordType_WmfRecordTypeCreateFontIndirect: EmfPlusRecordType = 66299;
pub const EmfPlusRecordType_WmfRecordTypeCreateBrushIndirect: EmfPlusRecordType = 66300;
pub const EmfPlusRecordType_WmfRecordTypeCreateBitmapIndirect: EmfPlusRecordType = 66301;
pub const EmfPlusRecordType_WmfRecordTypeCreateBitmap: EmfPlusRecordType = 67326;
pub const EmfPlusRecordType_WmfRecordTypeCreateRegion: EmfPlusRecordType = 67327;
pub const EmfPlusRecordType_EmfRecordTypeHeader: EmfPlusRecordType = 1;
pub const EmfPlusRecordType_EmfRecordTypePolyBezier: EmfPlusRecordType = 2;
pub const EmfPlusRecordType_EmfRecordTypePolygon: EmfPlusRecordType = 3;
pub const EmfPlusRecordType_EmfRecordTypePolyline: EmfPlusRecordType = 4;
pub const EmfPlusRecordType_EmfRecordTypePolyBezierTo: EmfPlusRecordType = 5;
pub const EmfPlusRecordType_EmfRecordTypePolyLineTo: EmfPlusRecordType = 6;
pub const EmfPlusRecordType_EmfRecordTypePolyPolyline: EmfPlusRecordType = 7;
pub const EmfPlusRecordType_EmfRecordTypePolyPolygon: EmfPlusRecordType = 8;
pub const EmfPlusRecordType_EmfRecordTypeSetWindowExtEx: EmfPlusRecordType = 9;
pub const EmfPlusRecordType_EmfRecordTypeSetWindowOrgEx: EmfPlusRecordType = 10;
pub const EmfPlusRecordType_EmfRecordTypeSetViewportExtEx: EmfPlusRecordType = 11;
pub const EmfPlusRecordType_EmfRecordTypeSetViewportOrgEx: EmfPlusRecordType = 12;
pub const EmfPlusRecordType_EmfRecordTypeSetBrushOrgEx: EmfPlusRecordType = 13;
pub const EmfPlusRecordType_EmfRecordTypeEOF: EmfPlusRecordType = 14;
pub const EmfPlusRecordType_EmfRecordTypeSetPixelV: EmfPlusRecordType = 15;
pub const EmfPlusRecordType_EmfRecordTypeSetMapperFlags: EmfPlusRecordType = 16;
pub const EmfPlusRecordType_EmfRecordTypeSetMapMode: EmfPlusRecordType = 17;
pub const EmfPlusRecordType_EmfRecordTypeSetBkMode: EmfPlusRecordType = 18;
pub const EmfPlusRecordType_EmfRecordTypeSetPolyFillMode: EmfPlusRecordType = 19;
pub const EmfPlusRecordType_EmfRecordTypeSetROP2: EmfPlusRecordType = 20;
pub const EmfPlusRecordType_EmfRecordTypeSetStretchBltMode: EmfPlusRecordType = 21;
pub const EmfPlusRecordType_EmfRecordTypeSetTextAlign: EmfPlusRecordType = 22;
pub const EmfPlusRecordType_EmfRecordTypeSetColorAdjustment: EmfPlusRecordType = 23;
pub const EmfPlusRecordType_EmfRecordTypeSetTextColor: EmfPlusRecordType = 24;
pub const EmfPlusRecordType_EmfRecordTypeSetBkColor: EmfPlusRecordType = 25;
pub const EmfPlusRecordType_EmfRecordTypeOffsetClipRgn: EmfPlusRecordType = 26;
pub const EmfPlusRecordType_EmfRecordTypeMoveToEx: EmfPlusRecordType = 27;
pub const EmfPlusRecordType_EmfRecordTypeSetMetaRgn: EmfPlusRecordType = 28;
pub const EmfPlusRecordType_EmfRecordTypeExcludeClipRect: EmfPlusRecordType = 29;
pub const EmfPlusRecordType_EmfRecordTypeIntersectClipRect: EmfPlusRecordType = 30;
pub const EmfPlusRecordType_EmfRecordTypeScaleViewportExtEx: EmfPlusRecordType = 31;
pub const EmfPlusRecordType_EmfRecordTypeScaleWindowExtEx: EmfPlusRecordType = 32;
pub const EmfPlusRecordType_EmfRecordTypeSaveDC: EmfPlusRecordType = 33;
pub const EmfPlusRecordType_EmfRecordTypeRestoreDC: EmfPlusRecordType = 34;
pub const EmfPlusRecordType_EmfRecordTypeSetWorldTransform: EmfPlusRecordType = 35;
pub const EmfPlusRecordType_EmfRecordTypeModifyWorldTransform: EmfPlusRecordType = 36;
pub const EmfPlusRecordType_EmfRecordTypeSelectObject: EmfPlusRecordType = 37;
pub const EmfPlusRecordType_EmfRecordTypeCreatePen: EmfPlusRecordType = 38;
pub const EmfPlusRecordType_EmfRecordTypeCreateBrushIndirect: EmfPlusRecordType = 39;
pub const EmfPlusRecordType_EmfRecordTypeDeleteObject: EmfPlusRecordType = 40;
pub const EmfPlusRecordType_EmfRecordTypeAngleArc: EmfPlusRecordType = 41;
pub const EmfPlusRecordType_EmfRecordTypeEllipse: EmfPlusRecordType = 42;
pub const EmfPlusRecordType_EmfRecordTypeRectangle: EmfPlusRecordType = 43;
pub const EmfPlusRecordType_EmfRecordTypeRoundRect: EmfPlusRecordType = 44;
pub const EmfPlusRecordType_EmfRecordTypeArc: EmfPlusRecordType = 45;
pub const EmfPlusRecordType_EmfRecordTypeChord: EmfPlusRecordType = 46;
pub const EmfPlusRecordType_EmfRecordTypePie: EmfPlusRecordType = 47;
pub const EmfPlusRecordType_EmfRecordTypeSelectPalette: EmfPlusRecordType = 48;
pub const EmfPlusRecordType_EmfRecordTypeCreatePalette: EmfPlusRecordType = 49;
pub const EmfPlusRecordType_EmfRecordTypeSetPaletteEntries: EmfPlusRecordType = 50;
pub const EmfPlusRecordType_EmfRecordTypeResizePalette: EmfPlusRecordType = 51;
pub const EmfPlusRecordType_EmfRecordTypeRealizePalette: EmfPlusRecordType = 52;
pub const EmfPlusRecordType_EmfRecordTypeExtFloodFill: EmfPlusRecordType = 53;
pub const EmfPlusRecordType_EmfRecordTypeLineTo: EmfPlusRecordType = 54;
pub const EmfPlusRecordType_EmfRecordTypeArcTo: EmfPlusRecordType = 55;
pub const EmfPlusRecordType_EmfRecordTypePolyDraw: EmfPlusRecordType = 56;
pub const EmfPlusRecordType_EmfRecordTypeSetArcDirection: EmfPlusRecordType = 57;
pub const EmfPlusRecordType_EmfRecordTypeSetMiterLimit: EmfPlusRecordType = 58;
pub const EmfPlusRecordType_EmfRecordTypeBeginPath: EmfPlusRecordType = 59;
pub const EmfPlusRecordType_EmfRecordTypeEndPath: EmfPlusRecordType = 60;
pub const EmfPlusRecordType_EmfRecordTypeCloseFigure: EmfPlusRecordType = 61;
pub const EmfPlusRecordType_EmfRecordTypeFillPath: EmfPlusRecordType = 62;
pub const EmfPlusRecordType_EmfRecordTypeStrokeAndFillPath: EmfPlusRecordType = 63;
pub const EmfPlusRecordType_EmfRecordTypeStrokePath: EmfPlusRecordType = 64;
pub const EmfPlusRecordType_EmfRecordTypeFlattenPath: EmfPlusRecordType = 65;
pub const EmfPlusRecordType_EmfRecordTypeWidenPath: EmfPlusRecordType = 66;
pub const EmfPlusRecordType_EmfRecordTypeSelectClipPath: EmfPlusRecordType = 67;
pub const EmfPlusRecordType_EmfRecordTypeAbortPath: EmfPlusRecordType = 68;
pub const EmfPlusRecordType_EmfRecordTypeReserved_069: EmfPlusRecordType = 69;
pub const EmfPlusRecordType_EmfRecordTypeGdiComment: EmfPlusRecordType = 70;
pub const EmfPlusRecordType_EmfRecordTypeFillRgn: EmfPlusRecordType = 71;
pub const EmfPlusRecordType_EmfRecordTypeFrameRgn: EmfPlusRecordType = 72;
pub const EmfPlusRecordType_EmfRecordTypeInvertRgn: EmfPlusRecordType = 73;
pub const EmfPlusRecordType_EmfRecordTypePaintRgn: EmfPlusRecordType = 74;
pub const EmfPlusRecordType_EmfRecordTypeExtSelectClipRgn: EmfPlusRecordType = 75;
pub const EmfPlusRecordType_EmfRecordTypeBitBlt: EmfPlusRecordType = 76;
pub const EmfPlusRecordType_EmfRecordTypeStretchBlt: EmfPlusRecordType = 77;
pub const EmfPlusRecordType_EmfRecordTypeMaskBlt: EmfPlusRecordType = 78;
pub const EmfPlusRecordType_EmfRecordTypePlgBlt: EmfPlusRecordType = 79;
pub const EmfPlusRecordType_EmfRecordTypeSetDIBitsToDevice: EmfPlusRecordType = 80;
pub const EmfPlusRecordType_EmfRecordTypeStretchDIBits: EmfPlusRecordType = 81;
pub const EmfPlusRecordType_EmfRecordTypeExtCreateFontIndirect: EmfPlusRecordType = 82;
pub const EmfPlusRecordType_EmfRecordTypeExtTextOutA: EmfPlusRecordType = 83;
pub const EmfPlusRecordType_EmfRecordTypeExtTextOutW: EmfPlusRecordType = 84;
pub const EmfPlusRecordType_EmfRecordTypePolyBezier16: EmfPlusRecordType = 85;
pub const EmfPlusRecordType_EmfRecordTypePolygon16: EmfPlusRecordType = 86;
pub const EmfPlusRecordType_EmfRecordTypePolyline16: EmfPlusRecordType = 87;
pub const EmfPlusRecordType_EmfRecordTypePolyBezierTo16: EmfPlusRecordType = 88;
pub const EmfPlusRecordType_EmfRecordTypePolylineTo16: EmfPlusRecordType = 89;
pub const EmfPlusRecordType_EmfRecordTypePolyPolyline16: EmfPlusRecordType = 90;
pub const EmfPlusRecordType_EmfRecordTypePolyPolygon16: EmfPlusRecordType = 91;
pub const EmfPlusRecordType_EmfRecordTypePolyDraw16: EmfPlusRecordType = 92;
pub const EmfPlusRecordType_EmfRecordTypeCreateMonoBrush: EmfPlusRecordType = 93;
pub const EmfPlusRecordType_EmfRecordTypeCreateDIBPatternBrushPt: EmfPlusRecordType = 94;
pub const EmfPlusRecordType_EmfRecordTypeExtCreatePen: EmfPlusRecordType = 95;
pub const EmfPlusRecordType_EmfRecordTypePolyTextOutA: EmfPlusRecordType = 96;
pub const EmfPlusRecordType_EmfRecordTypePolyTextOutW: EmfPlusRecordType = 97;
pub const EmfPlusRecordType_EmfRecordTypeSetICMMode: EmfPlusRecordType = 98;
pub const EmfPlusRecordType_EmfRecordTypeCreateColorSpace: EmfPlusRecordType = 99;
pub const EmfPlusRecordType_EmfRecordTypeSetColorSpace: EmfPlusRecordType = 100;
pub const EmfPlusRecordType_EmfRecordTypeDeleteColorSpace: EmfPlusRecordType = 101;
pub const EmfPlusRecordType_EmfRecordTypeGLSRecord: EmfPlusRecordType = 102;
pub const EmfPlusRecordType_EmfRecordTypeGLSBoundedRecord: EmfPlusRecordType = 103;
pub const EmfPlusRecordType_EmfRecordTypePixelFormat: EmfPlusRecordType = 104;
pub const EmfPlusRecordType_EmfRecordTypeDrawEscape: EmfPlusRecordType = 105;
pub const EmfPlusRecordType_EmfRecordTypeExtEscape: EmfPlusRecordType = 106;
pub const EmfPlusRecordType_EmfRecordTypeStartDoc: EmfPlusRecordType = 107;
pub const EmfPlusRecordType_EmfRecordTypeSmallTextOut: EmfPlusRecordType = 108;
pub const EmfPlusRecordType_EmfRecordTypeForceUFIMapping: EmfPlusRecordType = 109;
pub const EmfPlusRecordType_EmfRecordTypeNamedEscape: EmfPlusRecordType = 110;
pub const EmfPlusRecordType_EmfRecordTypeColorCorrectPalette: EmfPlusRecordType = 111;
pub const EmfPlusRecordType_EmfRecordTypeSetICMProfileA: EmfPlusRecordType = 112;
pub const EmfPlusRecordType_EmfRecordTypeSetICMProfileW: EmfPlusRecordType = 113;
pub const EmfPlusRecordType_EmfRecordTypeAlphaBlend: EmfPlusRecordType = 114;
pub const EmfPlusRecordType_EmfRecordTypeSetLayout: EmfPlusRecordType = 115;
pub const EmfPlusRecordType_EmfRecordTypeTransparentBlt: EmfPlusRecordType = 116;
pub const EmfPlusRecordType_EmfRecordTypeReserved_117: EmfPlusRecordType = 117;
pub const EmfPlusRecordType_EmfRecordTypeGradientFill: EmfPlusRecordType = 118;
pub const EmfPlusRecordType_EmfRecordTypeSetLinkedUFIs: EmfPlusRecordType = 119;
pub const EmfPlusRecordType_EmfRecordTypeSetTextJustification: EmfPlusRecordType = 120;
pub const EmfPlusRecordType_EmfRecordTypeColorMatchToTargetW: EmfPlusRecordType = 121;
pub const EmfPlusRecordType_EmfRecordTypeCreateColorSpaceW: EmfPlusRecordType = 122;
pub const EmfPlusRecordType_EmfRecordTypeMax: EmfPlusRecordType = 122;
pub const EmfPlusRecordType_EmfRecordTypeMin: EmfPlusRecordType = 1;
pub const EmfPlusRecordType_EmfPlusRecordTypeInvalid: EmfPlusRecordType = 16384;
pub const EmfPlusRecordType_EmfPlusRecordTypeHeader: EmfPlusRecordType = 16385;
pub const EmfPlusRecordType_EmfPlusRecordTypeEndOfFile: EmfPlusRecordType = 16386;
pub const EmfPlusRecordType_EmfPlusRecordTypeComment: EmfPlusRecordType = 16387;
pub const EmfPlusRecordType_EmfPlusRecordTypeGetDC: EmfPlusRecordType = 16388;
pub const EmfPlusRecordType_EmfPlusRecordTypeMultiFormatStart: EmfPlusRecordType = 16389;
pub const EmfPlusRecordType_EmfPlusRecordTypeMultiFormatSection: EmfPlusRecordType = 16390;
pub const EmfPlusRecordType_EmfPlusRecordTypeMultiFormatEnd: EmfPlusRecordType = 16391;
pub const EmfPlusRecordType_EmfPlusRecordTypeObject: EmfPlusRecordType = 16392;
pub const EmfPlusRecordType_EmfPlusRecordTypeClear: EmfPlusRecordType = 16393;
pub const EmfPlusRecordType_EmfPlusRecordTypeFillRects: EmfPlusRecordType = 16394;
pub const EmfPlusRecordType_EmfPlusRecordTypeDrawRects: EmfPlusRecordType = 16395;
pub const EmfPlusRecordType_EmfPlusRecordTypeFillPolygon: EmfPlusRecordType = 16396;
pub const EmfPlusRecordType_EmfPlusRecordTypeDrawLines: EmfPlusRecordType = 16397;
pub const EmfPlusRecordType_EmfPlusRecordTypeFillEllipse: EmfPlusRecordType = 16398;
pub const EmfPlusRecordType_EmfPlusRecordTypeDrawEllipse: EmfPlusRecordType = 16399;
pub const EmfPlusRecordType_EmfPlusRecordTypeFillPie: EmfPlusRecordType = 16400;
pub const EmfPlusRecordType_EmfPlusRecordTypeDrawPie: EmfPlusRecordType = 16401;
pub const EmfPlusRecordType_EmfPlusRecordTypeDrawArc: EmfPlusRecordType = 16402;
pub const EmfPlusRecordType_EmfPlusRecordTypeFillRegion: EmfPlusRecordType = 16403;
pub const EmfPlusRecordType_EmfPlusRecordTypeFillPath: EmfPlusRecordType = 16404;
pub const EmfPlusRecordType_EmfPlusRecordTypeDrawPath: EmfPlusRecordType = 16405;
pub const EmfPlusRecordType_EmfPlusRecordTypeFillClosedCurve: EmfPlusRecordType = 16406;
pub const EmfPlusRecordType_EmfPlusRecordTypeDrawClosedCurve: EmfPlusRecordType = 16407;
pub const EmfPlusRecordType_EmfPlusRecordTypeDrawCurve: EmfPlusRecordType = 16408;
pub const EmfPlusRecordType_EmfPlusRecordTypeDrawBeziers: EmfPlusRecordType = 16409;
pub const EmfPlusRecordType_EmfPlusRecordTypeDrawImage: EmfPlusRecordType = 16410;
pub const EmfPlusRecordType_EmfPlusRecordTypeDrawImagePoints: EmfPlusRecordType = 16411;
pub const EmfPlusRecordType_EmfPlusRecordTypeDrawString: EmfPlusRecordType = 16412;
pub const EmfPlusRecordType_EmfPlusRecordTypeSetRenderingOrigin: EmfPlusRecordType = 16413;
pub const EmfPlusRecordType_EmfPlusRecordTypeSetAntiAliasMode: EmfPlusRecordType = 16414;
pub const EmfPlusRecordType_EmfPlusRecordTypeSetTextRenderingHint: EmfPlusRecordType = 16415;
pub const EmfPlusRecordType_EmfPlusRecordTypeSetTextContrast: EmfPlusRecordType = 16416;
pub const EmfPlusRecordType_EmfPlusRecordTypeSetInterpolationMode: EmfPlusRecordType = 16417;
pub const EmfPlusRecordType_EmfPlusRecordTypeSetPixelOffsetMode: EmfPlusRecordType = 16418;
pub const EmfPlusRecordType_EmfPlusRecordTypeSetCompositingMode: EmfPlusRecordType = 16419;
pub const EmfPlusRecordType_EmfPlusRecordTypeSetCompositingQuality: EmfPlusRecordType = 16420;
pub const EmfPlusRecordType_EmfPlusRecordTypeSave: EmfPlusRecordType = 16421;
pub const EmfPlusRecordType_EmfPlusRecordTypeRestore: EmfPlusRecordType = 16422;
pub const EmfPlusRecordType_EmfPlusRecordTypeBeginContainer: EmfPlusRecordType = 16423;
pub const EmfPlusRecordType_EmfPlusRecordTypeBeginContainerNoParams: EmfPlusRecordType = 16424;
pub const EmfPlusRecordType_EmfPlusRecordTypeEndContainer: EmfPlusRecordType = 16425;
pub const EmfPlusRecordType_EmfPlusRecordTypeSetWorldTransform: EmfPlusRecordType = 16426;
pub const EmfPlusRecordType_EmfPlusRecordTypeResetWorldTransform: EmfPlusRecordType = 16427;
pub const EmfPlusRecordType_EmfPlusRecordTypeMultiplyWorldTransform: EmfPlusRecordType = 16428;
pub const EmfPlusRecordType_EmfPlusRecordTypeTranslateWorldTransform: EmfPlusRecordType = 16429;
pub const EmfPlusRecordType_EmfPlusRecordTypeScaleWorldTransform: EmfPlusRecordType = 16430;
pub const EmfPlusRecordType_EmfPlusRecordTypeRotateWorldTransform: EmfPlusRecordType = 16431;
pub const EmfPlusRecordType_EmfPlusRecordTypeSetPageTransform: EmfPlusRecordType = 16432;
pub const EmfPlusRecordType_EmfPlusRecordTypeResetClip: EmfPlusRecordType = 16433;
pub const EmfPlusRecordType_EmfPlusRecordTypeSetClipRect: EmfPlusRecordType = 16434;
pub const EmfPlusRecordType_EmfPlusRecordTypeSetClipPath: EmfPlusRecordType = 16435;
pub const EmfPlusRecordType_EmfPlusRecordTypeSetClipRegion: EmfPlusRecordType = 16436;
pub const EmfPlusRecordType_EmfPlusRecordTypeOffsetClip: EmfPlusRecordType = 16437;
pub const EmfPlusRecordType_EmfPlusRecordTypeDrawDriverString: EmfPlusRecordType = 16438;
pub const EmfPlusRecordType_EmfPlusRecordTotal: EmfPlusRecordType = 16439;
pub const EmfPlusRecordType_EmfPlusRecordTypeMax: EmfPlusRecordType = 16438;
pub const EmfPlusRecordType_EmfPlusRecordTypeMin: EmfPlusRecordType = 16385;
pub type EmfPlusRecordType = c_int;
pub const StringTrimming_StringTrimmingNone: StringTrimming = 0;
pub const StringTrimming_StringTrimmingCharacter: StringTrimming = 1;
pub const StringTrimming_StringTrimmingWord: StringTrimming = 2;
pub const StringTrimming_StringTrimmingEllipsisCharacter: StringTrimming = 3;
pub const StringTrimming_StringTrimmingEllipsisWord: StringTrimming = 4;
pub const StringTrimming_StringTrimmingEllipsisPath: StringTrimming = 5;
pub type StringTrimming = c_int;
pub const StringDigitSubstitute_StringDigitSubstituteUser: StringDigitSubstitute = 0;
pub const StringDigitSubstitute_StringDigitSubstituteNone: StringDigitSubstitute = 1;
pub const StringDigitSubstitute_StringDigitSubstituteNational: StringDigitSubstitute = 2;
pub const StringDigitSubstitute_StringDigitSubstituteTraditional: StringDigitSubstitute = 3;
pub type StringDigitSubstitute = c_int;
pub const StringAlignment_StringAlignmentNear: StringAlignment = 0;
pub const StringAlignment_StringAlignmentCenter: StringAlignment = 1;
pub const StringAlignment_StringAlignmentFar: StringAlignment = 2;
pub type StringAlignment = c_int;
pub const FlushIntention_FlushIntentionFlush: FlushIntention = 0;
pub const FlushIntention_FlushIntentionSync: FlushIntention = 1;
pub type FlushIntention = c_int;
pub const GpTestControlEnum_TestControlForceBilinear: GpTestControlEnum = 0;
pub const GpTestControlEnum_TestControlNoICM: GpTestControlEnum = 1;
pub const GpTestControlEnum_TestControlGetBuildNumber: GpTestControlEnum = 2;
pub type GpTestControlEnum = c_int;
pub type ImageAbort = Option<unsafe extern "C" fn(arg1: *mut c_void) -> BOOL>;
pub type DrawImageAbort = ImageAbort;
pub type GetThumbnailImageAbort = ImageAbort;
pub type EnumerateMetafileProc = Option<
    unsafe extern "C" fn(
        arg1: EmfPlusRecordType,
        arg2: UINT,
        arg3: UINT,
        arg4: *const BYTE,
        arg5: *mut c_void,
    ) -> BOOL,
>;
pub type REAL = f32;
pub const Status_Ok: Status = 0;
pub const Status_GenericError: Status = 1;
pub const Status_InvalidParameter: Status = 2;
pub const Status_OutOfMemory: Status = 3;
pub const Status_ObjectBusy: Status = 4;
pub const Status_InsufficientBuffer: Status = 5;
pub const Status_NotImplemented: Status = 6;
pub const Status_Win32Error: Status = 7;
pub const Status_WrongState: Status = 8;
pub const Status_Aborted: Status = 9;
pub const Status_FileNotFound: Status = 10;
pub const Status_ValueOverflow: Status = 11;
pub const Status_AccessDenied: Status = 12;
pub const Status_UnknownImageFormat: Status = 13;
pub const Status_FontFamilyNotFound: Status = 14;
pub const Status_FontStyleNotFound: Status = 15;
pub const Status_NotTrueTypeFont: Status = 16;
pub const Status_UnsupportedGdiplusVersion: Status = 17;
pub const Status_GdiplusNotInitialized: Status = 18;
pub const Status_PropertyNotFound: Status = 19;
pub const Status_PropertyNotSupported: Status = 20;
pub type Status = c_int;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct SizeF {
    pub Width: REAL,
    pub Height: REAL,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct PointF {
    pub X: REAL,
    pub Y: REAL,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Point {
    pub X: INT,
    pub Y: INT,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct RectF {
    pub X: REAL,
    pub Y: REAL,
    pub Width: REAL,
    pub Height: REAL,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Rect {
    pub X: INT,
    pub Y: INT,
    pub Width: INT,
    pub Height: INT,
}
#[repr(C)]
#[derive(Debug)]
pub struct PathData {
    pub Count: INT,
    pub Points: *mut PointF,
    pub Types: *mut BYTE,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct CharacterRange {
    pub First: INT,
    pub Length: INT,
}
pub const DebugEventLevel_DebugEventLevelFatal: DebugEventLevel = 0;
pub const DebugEventLevel_DebugEventLevelWarning: DebugEventLevel = 1;
pub type DebugEventLevel = c_int;
pub type DebugEventProc = Option<unsafe extern "C" fn(level: DebugEventLevel, message: *mut CHAR)>;
pub type NotificationHookProc = Option<unsafe extern "C" fn(token: *mut ULONG_PTR) -> Status>;
pub type NotificationUnhookProc = Option<unsafe extern "C" fn(token: ULONG_PTR)>;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct GdiplusStartupInput {
    pub GdiplusVersion: UINT32,
    pub DebugEventCallback: DebugEventProc,
    pub SuppressBackgroundThread: BOOL,
    pub SuppressExternalCodecs: BOOL,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct GdiplusStartupOutput {
    pub NotificationHook: NotificationHookProc,
    pub NotificationUnhook: NotificationUnhookProc,
}
extern "C" {
    #[link_name = "\u{1}GdiplusStartup"]
    pub fn GdiplusStartup(
        token: *mut ULONG_PTR,
        input: *const GdiplusStartupInput,
        output: *mut GdiplusStartupOutput,
    ) -> Status;
}
extern "C" {
    #[link_name = "\u{1}GdiplusShutdown"]
    pub fn GdiplusShutdown(token: ULONG_PTR);
}
pub type ARGB = DWORD;
pub type PixelFormat = INT;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ColorPalette {
    pub Flags: UINT,
    pub Count: UINT,
    pub Entries: [ARGB; 1usize],
}
pub const ColorChannelFlags_ColorChannelFlagsC: ColorChannelFlags = 0;
pub const ColorChannelFlags_ColorChannelFlagsM: ColorChannelFlags = 1;
pub const ColorChannelFlags_ColorChannelFlagsY: ColorChannelFlags = 2;
pub const ColorChannelFlags_ColorChannelFlagsK: ColorChannelFlags = 3;
pub const ColorChannelFlags_ColorChannelFlagsLast: ColorChannelFlags = 4;
pub type ColorChannelFlags = c_int;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Color {
    pub Argb: ARGB,
}
pub const Color_AliceBlue: c_int = -984833;
pub const Color_AntiqueWhite: c_int = -332841;
pub const Color_Aqua: c_int = -16711681;
pub const Color_Aquamarine: c_int = -8388652;
pub const Color_Azure: c_int = -983041;
pub const Color_Beige: c_int = -657956;
pub const Color_Bisque: c_int = -6972;
pub const Color_Black: c_int = -16777216;
pub const Color_BlanchedAlmond: c_int = -5171;
pub const Color_Blue: c_int = -16776961;
pub const Color_BlueViolet: c_int = -7722014;
pub const Color_Brown: c_int = -5952982;
pub const Color_BurlyWood: c_int = -2180985;
pub const Color_CadetBlue: c_int = -10510688;
pub const Color_Chartreuse: c_int = -8388864;
pub const Color_Chocolate: c_int = -2987746;
pub const Color_Coral: c_int = -32944;
pub const Color_CornflowerBlue: c_int = -10185235;
pub const Color_Cornsilk: c_int = -1828;
pub const Color_Crimson: c_int = -2354116;
pub const Color_Cyan: c_int = -16711681;
pub const Color_DarkBlue: c_int = -16777077;
pub const Color_DarkCyan: c_int = -16741493;
pub const Color_DarkGoldenrod: c_int = -4684277;
pub const Color_DarkGray: c_int = -5658199;
pub const Color_DarkGreen: c_int = -16751616;
pub const Color_DarkKhaki: c_int = -4343957;
pub const Color_DarkMagenta: c_int = -7667573;
pub const Color_DarkOliveGreen: c_int = -11179217;
pub const Color_DarkOrange: c_int = -29696;
pub const Color_DarkOrchid: c_int = -6737204;
pub const Color_DarkRed: c_int = -7667712;
pub const Color_DarkSalmon: c_int = -1468806;
pub const Color_DarkSeaGreen: c_int = -7357301;
pub const Color_DarkSlateBlue: c_int = -12042869;
pub const Color_DarkSlateGray: c_int = -13676721;
pub const Color_DarkTurquoise: c_int = -16724271;
pub const Color_DarkViolet: c_int = -7077677;
pub const Color_DeepPink: c_int = -60269;
pub const Color_DeepSkyBlue: c_int = -16728065;
pub const Color_DimGray: c_int = -9868951;
pub const Color_DodgerBlue: c_int = -14774017;
pub const Color_Firebrick: c_int = -5103070;
pub const Color_FloralWhite: c_int = -1296;
pub const Color_ForestGreen: c_int = -14513374;
pub const Color_Fuchsia: c_int = -65281;
pub const Color_Gainsboro: c_int = -2302756;
pub const Color_GhostWhite: c_int = -460545;
pub const Color_Gold: c_int = -10496;
pub const Color_Goldenrod: c_int = -2448096;
pub const Color_Gray: c_int = -8355712;
pub const Color_Green: c_int = -16744448;
pub const Color_GreenYellow: c_int = -5374161;
pub const Color_Honeydew: c_int = -983056;
pub const Color_HotPink: c_int = -38476;
pub const Color_IndianRed: c_int = -3318692;
pub const Color_Indigo: c_int = -11861886;
pub const Color_Ivory: c_int = -16;
pub const Color_Khaki: c_int = -989556;
pub const Color_Lavender: c_int = -1644806;
pub const Color_LavenderBlush: c_int = -3851;
pub const Color_LawnGreen: c_int = -8586240;
pub const Color_LemonChiffon: c_int = -1331;
pub const Color_LightBlue: c_int = -5383962;
pub const Color_LightCoral: c_int = -1015680;
pub const Color_LightCyan: c_int = -2031617;
pub const Color_LightGoldenrodYellow: c_int = -329006;
pub const Color_LightGray: c_int = -2894893;
pub const Color_LightGreen: c_int = -7278960;
pub const Color_LightPink: c_int = -18751;
pub const Color_LightSalmon: c_int = -24454;
pub const Color_LightSeaGreen: c_int = -14634326;
pub const Color_LightSkyBlue: c_int = -7876870;
pub const Color_LightSlateGray: c_int = -8943463;
pub const Color_LightSteelBlue: c_int = -5192482;
pub const Color_LightYellow: c_int = -32;
pub const Color_Lime: c_int = -16711936;
pub const Color_LimeGreen: c_int = -13447886;
pub const Color_Linen: c_int = -331546;
pub const Color_Magenta: c_int = -65281;
pub const Color_Maroon: c_int = -8388608;
pub const Color_MediumAquamarine: c_int = -10039894;
pub const Color_MediumBlue: c_int = -16777011;
pub const Color_MediumOrchid: c_int = -4565549;
pub const Color_MediumPurple: c_int = -7114533;
pub const Color_MediumSeaGreen: c_int = -12799119;
pub const Color_MediumSlateBlue: c_int = -8689426;
pub const Color_MediumSpringGreen: c_int = -16713062;
pub const Color_MediumTurquoise: c_int = -12004916;
pub const Color_MediumVioletRed: c_int = -3730043;
pub const Color_MidnightBlue: c_int = -15132304;
pub const Color_MintCream: c_int = -655366;
pub const Color_MistyRose: c_int = -6943;
pub const Color_Moccasin: c_int = -6987;
pub const Color_NavajoWhite: c_int = -8531;
pub const Color_Navy: c_int = -16777088;
pub const Color_OldLace: c_int = -133658;
pub const Color_Olive: c_int = -8355840;
pub const Color_OliveDrab: c_int = -9728477;
pub const Color_Orange: c_int = -23296;
pub const Color_OrangeRed: c_int = -47872;
pub const Color_Orchid: c_int = -2461482;
pub const Color_PaleGoldenrod: c_int = -1120086;
pub const Color_PaleGreen: c_int = -6751336;
pub const Color_PaleTurquoise: c_int = -5247250;
pub const Color_PaleVioletRed: c_int = -2396013;
pub const Color_PapayaWhip: c_int = -4139;
pub const Color_PeachPuff: c_int = -9543;
pub const Color_Peru: c_int = -3308225;
pub const Color_Pink: c_int = -16181;
pub const Color_Plum: c_int = -2252579;
pub const Color_PowderBlue: c_int = -5185306;
pub const Color_Purple: c_int = -8388480;
pub const Color_Red: c_int = -65536;
pub const Color_RosyBrown: c_int = -4419697;
pub const Color_RoyalBlue: c_int = -12490271;
pub const Color_SaddleBrown: c_int = -7650029;
pub const Color_Salmon: c_int = -360334;
pub const Color_SandyBrown: c_int = -744352;
pub const Color_SeaGreen: c_int = -13726889;
pub const Color_SeaShell: c_int = -2578;
pub const Color_Sienna: c_int = -6270419;
pub const Color_Silver: c_int = -4144960;
pub const Color_SkyBlue: c_int = -7876885;
pub const Color_SlateBlue: c_int = -9807155;
pub const Color_SlateGray: c_int = -9404272;
pub const Color_Snow: c_int = -1286;
pub const Color_SpringGreen: c_int = -16711809;
pub const Color_SteelBlue: c_int = -12156236;
pub const Color_Tan: c_int = -2968436;
pub const Color_Teal: c_int = -16744320;
pub const Color_Thistle: c_int = -2572328;
pub const Color_Tomato: c_int = -40121;
pub const Color_Transparent: c_int = 16777215;
pub const Color_Turquoise: c_int = -12525360;
pub const Color_Violet: c_int = -1146130;
pub const Color_Wheat: c_int = -663885;
pub const Color_White: c_int = -1;
pub const Color_WhiteSmoke: c_int = -657931;
pub const Color_Yellow: c_int = -256;
pub const Color_YellowGreen: c_int = -6632142;
pub type Color__bindgen_ty_1 = c_int;
pub const Color_AlphaShift: c_int = 24;
pub const Color_RedShift: c_int = 16;
pub const Color_GreenShift: c_int = 8;
pub const Color_BlueShift: c_int = 0;
pub type Color__bindgen_ty_2 = c_int;
pub const Color_AlphaMask: c_int = -16777216;
pub const Color_RedMask: c_int = 16711680;
pub const Color_GreenMask: c_int = 65280;
pub const Color_BlueMask: c_int = 255;
pub type Color__bindgen_ty_3 = c_int;
#[repr(C)]
#[derive(Copy, Clone)]
pub struct ENHMETAHEADER3 {
    pub iType: DWORD,
    pub nSize: DWORD,
    pub rclBounds: RECTL,
    pub rclFrame: RECTL,
    pub dSignature: DWORD,
    pub nVersion: DWORD,
    pub nBytes: DWORD,
    pub nRecords: DWORD,
    pub nHandles: WORD,
    pub sReserved: WORD,
    pub nDescription: DWORD,
    pub offDescription: DWORD,
    pub nPalEntries: DWORD,
    pub szlDevice: SIZEL,
    pub szlMillimeters: SIZEL,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct PWMFRect16 {
    pub Left: INT16,
    pub Top: INT16,
    pub Right: INT16,
    pub Bottom: INT16,
}
#[repr(C, packed(2))]
#[derive(Debug, Copy, Clone)]
pub struct WmfPlaceableFileHeader {
    pub Key: UINT32,
    pub Hmf: INT16,
    pub BoundingBox: PWMFRect16,
    pub Inch: INT16,
    pub Reserved: UINT32,
    pub Checksum: INT16,
}
#[repr(C)]
#[derive(Copy, Clone)]
pub struct MetafileHeader {
    pub Type: MetafileType,
    pub Size: UINT,
    pub Version: UINT,
    pub EmfPlusFlags: UINT,
    pub DpiX: REAL,
    pub DpiY: REAL,
    pub X: INT,
    pub Y: INT,
    pub Width: INT,
    pub Height: INT,
    pub __bindgen_anon_1: MetafileHeader__bindgen_ty_1,
    pub EmfPlusHeaderSize: INT,
    pub LogicalDpiX: INT,
    pub LogicalDpiY: INT,
}
#[repr(C)]
#[derive(Copy, Clone)]
pub union MetafileHeader__bindgen_ty_1 {
    pub WmfHeader: METAHEADER,
    pub EmfHeader: ENHMETAHEADER3,
}
#[repr(C)]
#[derive(Copy, Clone)]
pub struct ImageCodecInfo {
    pub Clsid: CLSID,
    pub FormatID: GUID,
    pub CodecName: *const WCHAR,
    pub DllName: *const WCHAR,
    pub FormatDescription: *const WCHAR,
    pub FilenameExtension: *const WCHAR,
    pub MimeType: *const WCHAR,
    pub Flags: DWORD,
    pub Version: DWORD,
    pub SigCount: DWORD,
    pub SigSize: DWORD,
    pub SigPattern: *const BYTE,
    pub SigMask: *const BYTE,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct BitmapData {
    pub Width: UINT,
    pub Height: UINT,
    pub Stride: INT,
    pub PixelFormat: PixelFormat,
    pub Scan0: *mut c_void,
    pub Reserved: UINT_PTR,
}
pub const RotateFlipType_RotateNoneFlipNone: RotateFlipType = 0;
pub const RotateFlipType_Rotate90FlipNone: RotateFlipType = 1;
pub const RotateFlipType_Rotate180FlipNone: RotateFlipType = 2;
pub const RotateFlipType_Rotate270FlipNone: RotateFlipType = 3;
pub const RotateFlipType_RotateNoneFlipX: RotateFlipType = 4;
pub const RotateFlipType_Rotate90FlipX: RotateFlipType = 5;
pub const RotateFlipType_Rotate180FlipX: RotateFlipType = 6;
pub const RotateFlipType_Rotate270FlipX: RotateFlipType = 7;
pub const RotateFlipType_RotateNoneFlipY: RotateFlipType = 6;
pub const RotateFlipType_Rotate90FlipY: RotateFlipType = 7;
pub const RotateFlipType_Rotate180FlipY: RotateFlipType = 4;
pub const RotateFlipType_Rotate270FlipY: RotateFlipType = 5;
pub const RotateFlipType_RotateNoneFlipXY: RotateFlipType = 2;
pub const RotateFlipType_Rotate90FlipXY: RotateFlipType = 3;
pub const RotateFlipType_Rotate180FlipXY: RotateFlipType = 0;
pub const RotateFlipType_Rotate270FlipXY: RotateFlipType = 1;
pub type RotateFlipType = c_int;
#[repr(C)]
#[derive(Copy, Clone)]
pub struct EncoderParameter {
    pub Guid: GUID,
    pub NumberOfValues: ULONG,
    pub Type: ULONG,
    pub Value: *mut c_void,
}
#[repr(C)]
#[derive(Copy, Clone)]
pub struct EncoderParameters {
    pub Count: UINT,
    pub Parameter: [EncoderParameter; 1usize],
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct PropertyItem {
    pub id: PROPID,
    pub length: ULONG,
    pub type_: WORD,
    pub value: *mut c_void,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ColorMatrix {
    pub m: [[REAL; 5usize]; 5usize],
}
pub const ColorMatrixFlags_ColorMatrixFlagsDefault: ColorMatrixFlags = 0;
pub const ColorMatrixFlags_ColorMatrixFlagsSkipGrays: ColorMatrixFlags = 1;
pub const ColorMatrixFlags_ColorMatrixFlagsAltGray: ColorMatrixFlags = 2;
pub type ColorMatrixFlags = c_int;
pub const ColorAdjustType_ColorAdjustTypeDefault: ColorAdjustType = 0;
pub const ColorAdjustType_ColorAdjustTypeBitmap: ColorAdjustType = 1;
pub const ColorAdjustType_ColorAdjustTypeBrush: ColorAdjustType = 2;
pub const ColorAdjustType_ColorAdjustTypePen: ColorAdjustType = 3;
pub const ColorAdjustType_ColorAdjustTypeText: ColorAdjustType = 4;
pub const ColorAdjustType_ColorAdjustTypeCount: ColorAdjustType = 5;
pub const ColorAdjustType_ColorAdjustTypeAny: ColorAdjustType = 6;
pub type ColorAdjustType = c_int;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ColorMap {
    pub oldColor: Color,
    pub newColor: Color,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct GpGraphics {
    pub _address: u8,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct GpBrush {
    pub _address: u8,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct GpTexture {
    pub _address: u8,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct GpSolidFill {
    pub _address: u8,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct GpLineGradient {
    pub _address: u8,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct GpPathGradient {
    pub _address: u8,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct GpHatch {
    pub _address: u8,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct GpPen {
    pub _address: u8,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct GpCustomLineCap {
    pub _address: u8,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct GpAdjustableArrowCap {
    pub _address: u8,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct GpImage {
    pub _address: u8,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct GpBitmap {
    pub _address: u8,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct GpMetafile {
    pub _address: u8,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct GpImageAttributes {
    pub _address: u8,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct GpPath {
    pub _address: u8,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct GpRegion {
    pub _address: u8,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct GpPathIterator {
    pub _address: u8,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct GpFontFamily {
    pub _address: u8,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct GpFont {
    pub _address: u8,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct GpStringFormat {
    pub _address: u8,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct GpFontCollection {
    pub _address: u8,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct GpCachedBitmap {
    _unused: [u8; 0],
}
pub use self::{
    CoordinateSpace as GpCoordinateSpace, FillMode as GpFillMode, Status as GpStatus,
    Unit as GpUnit, WrapMode as GpWrapMode,
};
pub type GpPointF = PointF;
pub type GpPoint = Point;
pub type GpRectF = RectF;
pub type GpRect = Rect;
pub use self::{
    DashCap as GpDashCap, DashStyle as GpDashStyle, HatchStyle as GpHatchStyle,
    LineCap as GpLineCap, LineJoin as GpLineJoin, PenAlignment as GpPenAlignment,
    PenType as GpPenType,
};
pub type GpMatrix = Matrix;
pub use self::{
    BrushType as GpBrushType, FlushIntention as GpFlushIntention, MatrixOrder as GpMatrixOrder,
};
pub type GpPathData = PathData;
#[repr(C)]
#[derive(Debug)]
pub struct Region {
    pub nativeRegion: *mut GpRegion,
    pub lastResult: Status,
}
extern "C" {
    #[link_name = "\u{1}?FromHRGN@Region@Gdiplus@@SAPEAV12@PEAUHRGN__@@@Z"]
    pub fn Region_FromHRGN(hRgn: HRGN) -> *mut Region;
}
extern "C" {
    #[link_name = "\u{1}?Clone@Region@Gdiplus@@QEBAPEAV12@XZ"]
    pub fn Region_Clone(this: *const Region) -> *mut Region;
}
extern "C" {
    #[link_name = "\u{1}?MakeInfinite@Region@Gdiplus@@QEAA?AW4Status@2@XZ"]
    pub fn Region_MakeInfinite(this: *mut Region) -> Status;
}
extern "C" {
    #[link_name = "\u{1}?MakeEmpty@Region@Gdiplus@@QEAA?AW4Status@2@XZ"]
    pub fn Region_MakeEmpty(this: *mut Region) -> Status;
}
extern "C" {
    #[link_name = "\u{1}?GetDataSize@Region@Gdiplus@@QEBAIXZ"]
    pub fn Region_GetDataSize(this: *const Region) -> UINT;
}
extern "C" {
    #[link_name = "\u{1}?GetData@Region@Gdiplus@@QEBA?AW4Status@2@PEAEIPEAI@Z"]
    pub fn Region_GetData(
        this: *const Region,
        buffer: *mut BYTE,
        bufferSize: UINT,
        sizeFilled: *mut UINT,
    ) -> Status;
}
extern "C" {
    #[link_name = "\u{1}?Intersect@Region@Gdiplus@@QEAA?AW4Status@2@AEBVRect@2@@Z"]
    pub fn Region_Intersect(this: *mut Region, rect: *const Rect) -> Status;
}
extern "C" {
    #[link_name = "\u{1}?Intersect@Region@Gdiplus@@QEAA?AW4Status@2@AEBVRectF@2@@Z"]
    pub fn Region_Intersect1(this: *mut Region, rect: *const RectF) -> Status;
}
extern "C" {
    #[link_name = "\u{1}?Intersect@Region@Gdiplus@@QEAA?AW4Status@2@PEBVGraphicsPath@2@@Z"]
    pub fn Region_Intersect2(this: *mut Region, path: *const GraphicsPath) -> Status;
}
extern "C" {
    #[link_name = "\u{1}?Intersect@Region@Gdiplus@@QEAA?AW4Status@2@PEBV12@@Z"]
    pub fn Region_Intersect3(this: *mut Region, region: *const Region) -> Status;
}
extern "C" {
    #[link_name = "\u{1}?Union@Region@Gdiplus@@QEAA?AW4Status@2@AEBVRect@2@@Z"]
    pub fn Region_Union(this: *mut Region, rect: *const Rect) -> Status;
}
extern "C" {
    #[link_name = "\u{1}?Union@Region@Gdiplus@@QEAA?AW4Status@2@AEBVRectF@2@@Z"]
    pub fn Region_Union1(this: *mut Region, rect: *const RectF) -> Status;
}
extern "C" {
    #[link_name = "\u{1}?Union@Region@Gdiplus@@QEAA?AW4Status@2@PEBVGraphicsPath@2@@Z"]
    pub fn Region_Union2(this: *mut Region, path: *const GraphicsPath) -> Status;
}
extern "C" {
    #[link_name = "\u{1}?Union@Region@Gdiplus@@QEAA?AW4Status@2@PEBV12@@Z"]
    pub fn Region_Union3(this: *mut Region, region: *const Region) -> Status;
}
extern "C" {
    #[link_name = "\u{1}?Xor@Region@Gdiplus@@QEAA?AW4Status@2@AEBVRect@2@@Z"]
    pub fn Region_Xor(this: *mut Region, rect: *const Rect) -> Status;
}
extern "C" {
    #[link_name = "\u{1}?Xor@Region@Gdiplus@@QEAA?AW4Status@2@AEBVRectF@2@@Z"]
    pub fn Region_Xor1(this: *mut Region, rect: *const RectF) -> Status;
}
extern "C" {
    #[link_name = "\u{1}?Xor@Region@Gdiplus@@QEAA?AW4Status@2@PEBVGraphicsPath@2@@Z"]
    pub fn Region_Xor2(this: *mut Region, path: *const GraphicsPath) -> Status;
}
extern "C" {
    #[link_name = "\u{1}?Xor@Region@Gdiplus@@QEAA?AW4Status@2@PEBV12@@Z"]
    pub fn Region_Xor3(this: *mut Region, region: *const Region) -> Status;
}
extern "C" {
    #[link_name = "\u{1}?Exclude@Region@Gdiplus@@QEAA?AW4Status@2@AEBVRect@2@@Z"]
    pub fn Region_Exclude(this: *mut Region, rect: *const Rect) -> Status;
}
extern "C" {
    #[link_name = "\u{1}?Exclude@Region@Gdiplus@@QEAA?AW4Status@2@AEBVRectF@2@@Z"]
    pub fn Region_Exclude1(this: *mut Region, rect: *const RectF) -> Status;
}
extern "C" {
    #[link_name = "\u{1}?Exclude@Region@Gdiplus@@QEAA?AW4Status@2@PEBVGraphicsPath@2@@Z"]
    pub fn Region_Exclude2(this: *mut Region, path: *const GraphicsPath) -> Status;
}
extern "C" {
    #[link_name = "\u{1}?Exclude@Region@Gdiplus@@QEAA?AW4Status@2@PEBV12@@Z"]
    pub fn Region_Exclude3(this: *mut Region, region: *const Region) -> Status;
}
extern "C" {
    #[link_name = "\u{1}?Complement@Region@Gdiplus@@QEAA?AW4Status@2@AEBVRect@2@@Z"]
    pub fn Region_Complement(this: *mut Region, rect: *const Rect) -> Status;
}
extern "C" {
    #[link_name = "\u{1}?Complement@Region@Gdiplus@@QEAA?AW4Status@2@AEBVRectF@2@@Z"]
    pub fn Region_Complement1(this: *mut Region, rect: *const RectF) -> Status;
}
extern "C" {
    #[link_name = "\u{1}?Complement@Region@Gdiplus@@QEAA?AW4Status@2@PEBVGraphicsPath@2@@Z"]
    pub fn Region_Complement2(this: *mut Region, path: *const GraphicsPath) -> Status;
}
extern "C" {
    #[link_name = "\u{1}?Complement@Region@Gdiplus@@QEAA?AW4Status@2@PEBV12@@Z"]
    pub fn Region_Complement3(this: *mut Region, region: *const Region) -> Status;
}
extern "C" {
    #[link_name = "\u{1}?Translate@Region@Gdiplus@@QEAA?AW4Status@2@MM@Z"]
    pub fn Region_Translate(this: *mut Region, dx: REAL, dy: REAL) -> Status;
}
extern "C" {
    #[link_name = "\u{1}?Translate@Region@Gdiplus@@QEAA?AW4Status@2@HH@Z"]
    pub fn Region_Translate1(this: *mut Region, dx: INT, dy: INT) -> Status;
}
extern "C" {
    #[link_name = "\u{1}?Transform@Region@Gdiplus@@QEAA?AW4Status@2@PEBVMatrix@2@@Z"]
    pub fn Region_Transform(this: *mut Region, matrix: *const Matrix) -> Status;
}
extern "C" {
    #[link_name = "\u{1}?GetBounds@Region@Gdiplus@@QEBA?AW4Status@2@PEAVRect@2@PEBVGraphics@2@@Z"]
    pub fn Region_GetBounds(this: *const Region, rect: *mut Rect, g: *const Graphics) -> Status;
}
extern "C" {
    #[link_name = "\u{1}?GetBounds@Region@Gdiplus@@QEBA?AW4Status@2@PEAVRectF@2@PEBVGraphics@2@@Z"]
    pub fn Region_GetBounds1(this: *const Region, rect: *mut RectF, g: *const Graphics) -> Status;
}
extern "C" {
    #[link_name = "\u{1}?GetHRGN@Region@Gdiplus@@QEBAPEAUHRGN__@@PEBVGraphics@2@@Z"]
    pub fn Region_GetHRGN(this: *const Region, g: *const Graphics) -> HRGN;
}
extern "C" {
    #[link_name = "\u{1}?IsEmpty@Region@Gdiplus@@QEBAHPEBVGraphics@2@@Z"]
    pub fn Region_IsEmpty(this: *const Region, g: *const Graphics) -> BOOL;
}
extern "C" {
    #[link_name = "\u{1}?IsInfinite@Region@Gdiplus@@QEBAHPEBVGraphics@2@@Z"]
    pub fn Region_IsInfinite(this: *const Region, g: *const Graphics) -> BOOL;
}
extern "C" {
    #[link_name = "\u{1}?IsVisible@Region@Gdiplus@@QEBAHAEBVPoint@2@PEBVGraphics@2@@Z"]
    pub fn Region_IsVisible(this: *const Region, point: *const Point, g: *const Graphics) -> BOOL;
}
extern "C" {
    #[link_name = "\u{1}?IsVisible@Region@Gdiplus@@QEBAHAEBVPointF@2@PEBVGraphics@2@@Z"]
    pub fn Region_IsVisible1(this: *const Region, point: *const PointF, g: *const Graphics)
        -> BOOL;
}
extern "C" {
    #[link_name = "\u{1}?IsVisible@Region@Gdiplus@@QEBAHAEBVRect@2@PEBVGraphics@2@@Z"]
    pub fn Region_IsVisible2(this: *const Region, rect: *const Rect, g: *const Graphics) -> BOOL;
}
extern "C" {
    #[link_name = "\u{1}?IsVisible@Region@Gdiplus@@QEBAHAEBVRectF@2@PEBVGraphics@2@@Z"]
    pub fn Region_IsVisible3(this: *const Region, rect: *const RectF, g: *const Graphics) -> BOOL;
}
extern "C" {
    #[link_name = "\u{1}?Equals@Region@Gdiplus@@QEBAHPEBV12@PEBVGraphics@2@@Z"]
    pub fn Region_Equals(this: *const Region, region: *const Region, g: *const Graphics) -> BOOL;
}
extern "C" {
    #[link_name = "\u{1}?GetRegionScansCount@Region@Gdiplus@@QEBAIPEBVMatrix@2@@Z"]
    pub fn Region_GetRegionScansCount(this: *const Region, matrix: *const Matrix) -> UINT;
}
extern "C" {
    #[link_name = "\u{1}?GetRegionScans@Region@Gdiplus@@QEBA?AW4Status@2@PEBVMatrix@2@PEAVRectF@2@PEAH@Z"]
    pub fn Region_GetRegionScans(
        this: *const Region,
        matrix: *const Matrix,
        rects: *mut RectF,
        count: *mut INT,
    ) -> Status;
}
extern "C" {
    #[link_name = "\u{1}?GetRegionScans@Region@Gdiplus@@QEBA?AW4Status@2@PEBVMatrix@2@PEAVRect@2@PEAH@Z"]
    pub fn Region_GetRegionScans1(
        this: *const Region,
        matrix: *const Matrix,
        rects: *mut Rect,
        count: *mut INT,
    ) -> Status;
}
extern "C" {
    #[link_name = "\u{1}?GetLastStatus@Region@Gdiplus@@QEBA?AW4Status@2@XZ"]
    pub fn Region_GetLastStatus(this: *const Region) -> Status;
}
extern "C" {
    #[link_name = "\u{1}?SetNativeRegion@Region@Gdiplus@@IEAAXPEAVGpRegion@2@@Z"]
    pub fn Region_SetNativeRegion(this: *mut Region, nativeRegion: *mut GpRegion);
}
extern "C" {
    #[link_name = "\u{1}??0Region@Gdiplus@@QEAA@XZ"]
    pub fn Region_Region(this: *mut Region);
}
extern "C" {
    #[link_name = "\u{1}??0Region@Gdiplus@@QEAA@AEBVRectF@1@@Z"]
    pub fn Region_Region1(this: *mut Region, rect: *const RectF);
}
extern "C" {
    #[link_name = "\u{1}??0Region@Gdiplus@@QEAA@AEBVRect@1@@Z"]
    pub fn Region_Region2(this: *mut Region, rect: *const Rect);
}
extern "C" {
    #[link_name = "\u{1}??0Region@Gdiplus@@QEAA@PEBVGraphicsPath@1@@Z"]
    pub fn Region_Region3(this: *mut Region, path: *const GraphicsPath);
}
extern "C" {
    #[link_name = "\u{1}??0Region@Gdiplus@@QEAA@PEBEH@Z"]
    pub fn Region_Region4(this: *mut Region, regionData: *const BYTE, size: INT);
}
extern "C" {
    #[link_name = "\u{1}??0Region@Gdiplus@@QEAA@PEAUHRGN__@@@Z"]
    pub fn Region_Region5(this: *mut Region, hRgn: HRGN);
}
extern "C" {
    #[link_name = "\u{1}??0Region@Gdiplus@@IEAA@PEAVGpRegion@1@@Z"]
    pub fn Region_Region6(this: *mut Region, nativeRegion: *mut GpRegion);
}
extern "C" {
    #[link_name = "\u{1}??_DRegion@Gdiplus@@QEAAXXZ"]
    pub fn Region_Region_destructor(this: *mut Region);
}
impl Region {
    #[inline]
    pub unsafe fn FromHRGN(hRgn: HRGN) -> *mut Region {
        Region_FromHRGN(hRgn)
    }
    #[inline]
    pub unsafe fn Clone(&self) -> *mut Region {
        Region_Clone(self)
    }
    #[inline]
    pub unsafe fn MakeInfinite(&mut self) -> Status {
        Region_MakeInfinite(self)
    }
    #[inline]
    pub unsafe fn MakeEmpty(&mut self) -> Status {
        Region_MakeEmpty(self)
    }
    #[inline]
    pub unsafe fn GetDataSize(&self) -> UINT {
        Region_GetDataSize(self)
    }
    #[inline]
    pub unsafe fn GetData(
        &self,
        buffer: *mut BYTE,
        bufferSize: UINT,
        sizeFilled: *mut UINT,
    ) -> Status {
        Region_GetData(self, buffer, bufferSize, sizeFilled)
    }
    #[inline]
    pub unsafe fn Intersect(&mut self, rect: *const Rect) -> Status {
        Region_Intersect(self, rect)
    }
    #[inline]
    pub unsafe fn Intersect1(&mut self, rect: *const RectF) -> Status {
        Region_Intersect1(self, rect)
    }
    #[inline]
    pub unsafe fn Intersect2(&mut self, path: *const GraphicsPath) -> Status {
        Region_Intersect2(self, path)
    }
    #[inline]
    pub unsafe fn Intersect3(&mut self, region: *const Region) -> Status {
        Region_Intersect3(self, region)
    }
    #[inline]
    pub unsafe fn Union(&mut self, rect: *const Rect) -> Status {
        Region_Union(self, rect)
    }
    #[inline]
    pub unsafe fn Union1(&mut self, rect: *const RectF) -> Status {
        Region_Union1(self, rect)
    }
    #[inline]
    pub unsafe fn Union2(&mut self, path: *const GraphicsPath) -> Status {
        Region_Union2(self, path)
    }
    #[inline]
    pub unsafe fn Union3(&mut self, region: *const Region) -> Status {
        Region_Union3(self, region)
    }
    #[inline]
    pub unsafe fn Xor(&mut self, rect: *const Rect) -> Status {
        Region_Xor(self, rect)
    }
    #[inline]
    pub unsafe fn Xor1(&mut self, rect: *const RectF) -> Status {
        Region_Xor1(self, rect)
    }
    #[inline]
    pub unsafe fn Xor2(&mut self, path: *const GraphicsPath) -> Status {
        Region_Xor2(self, path)
    }
    #[inline]
    pub unsafe fn Xor3(&mut self, region: *const Region) -> Status {
        Region_Xor3(self, region)
    }
    #[inline]
    pub unsafe fn Exclude(&mut self, rect: *const Rect) -> Status {
        Region_Exclude(self, rect)
    }
    #[inline]
    pub unsafe fn Exclude1(&mut self, rect: *const RectF) -> Status {
        Region_Exclude1(self, rect)
    }
    #[inline]
    pub unsafe fn Exclude2(&mut self, path: *const GraphicsPath) -> Status {
        Region_Exclude2(self, path)
    }
    #[inline]
    pub unsafe fn Exclude3(&mut self, region: *const Region) -> Status {
        Region_Exclude3(self, region)
    }
    #[inline]
    pub unsafe fn Complement(&mut self, rect: *const Rect) -> Status {
        Region_Complement(self, rect)
    }
    #[inline]
    pub unsafe fn Complement1(&mut self, rect: *const RectF) -> Status {
        Region_Complement1(self, rect)
    }
    #[inline]
    pub unsafe fn Complement2(&mut self, path: *const GraphicsPath) -> Status {
        Region_Complement2(self, path)
    }
    #[inline]
    pub unsafe fn Complement3(&mut self, region: *const Region) -> Status {
        Region_Complement3(self, region)
    }
    #[inline]
    pub unsafe fn Translate(&mut self, dx: REAL, dy: REAL) -> Status {
        Region_Translate(self, dx, dy)
    }
    #[inline]
    pub unsafe fn Translate1(&mut self, dx: INT, dy: INT) -> Status {
        Region_Translate1(self, dx, dy)
    }
    #[inline]
    pub unsafe fn Transform(&mut self, matrix: *const Matrix) -> Status {
        Region_Transform(self, matrix)
    }
    #[inline]
    pub unsafe fn GetBounds(&self, rect: *mut Rect, g: *const Graphics) -> Status {
        Region_GetBounds(self, rect, g)
    }
    #[inline]
    pub unsafe fn GetBounds1(&self, rect: *mut RectF, g: *const Graphics) -> Status {
        Region_GetBounds1(self, rect, g)
    }
    #[inline]
    pub unsafe fn GetHRGN(&self, g: *const Graphics) -> HRGN {
        Region_GetHRGN(self, g)
    }
    #[inline]
    pub unsafe fn IsEmpty(&self, g: *const Graphics) -> BOOL {
        Region_IsEmpty(self, g)
    }
    #[inline]
    pub unsafe fn IsInfinite(&self, g: *const Graphics) -> BOOL {
        Region_IsInfinite(self, g)
    }
    #[inline]
    pub unsafe fn IsVisible(&self, point: *const Point, g: *const Graphics) -> BOOL {
        Region_IsVisible(self, point, g)
    }
    #[inline]
    pub unsafe fn IsVisible1(&self, point: *const PointF, g: *const Graphics) -> BOOL {
        Region_IsVisible1(self, point, g)
    }
    #[inline]
    pub unsafe fn IsVisible2(&self, rect: *const Rect, g: *const Graphics) -> BOOL {
        Region_IsVisible2(self, rect, g)
    }
    #[inline]
    pub unsafe fn IsVisible3(&self, rect: *const RectF, g: *const Graphics) -> BOOL {
        Region_IsVisible3(self, rect, g)
    }
    #[inline]
    pub unsafe fn Equals(&self, region: *const Region, g: *const Graphics) -> BOOL {
        Region_Equals(self, region, g)
    }
    #[inline]
    pub unsafe fn GetRegionScansCount(&self, matrix: *const Matrix) -> UINT {
        Region_GetRegionScansCount(self, matrix)
    }
    #[inline]
    pub unsafe fn GetRegionScans(
        &self,
        matrix: *const Matrix,
        rects: *mut RectF,
        count: *mut INT,
    ) -> Status {
        Region_GetRegionScans(self, matrix, rects, count)
    }
    #[inline]
    pub unsafe fn GetRegionScans1(
        &self,
        matrix: *const Matrix,
        rects: *mut Rect,
        count: *mut INT,
    ) -> Status {
        Region_GetRegionScans1(self, matrix, rects, count)
    }
    #[inline]
    pub unsafe fn GetLastStatus(&self) -> Status {
        Region_GetLastStatus(self)
    }
    #[inline]
    pub unsafe fn SetNativeRegion(&mut self, nativeRegion: *mut GpRegion) {
        Region_SetNativeRegion(self, nativeRegion)
    }
    #[inline]
    pub unsafe fn new() -> Self {
        let mut __bindgen_tmp = MaybeUninit::uninit();
        Region_Region(__bindgen_tmp.as_mut_ptr());
        __bindgen_tmp.assume_init()
    }
    #[inline]
    pub unsafe fn new1(rect: *const RectF) -> Self {
        let mut __bindgen_tmp = MaybeUninit::uninit();
        Region_Region1(__bindgen_tmp.as_mut_ptr(), rect);
        __bindgen_tmp.assume_init()
    }
    #[inline]
    pub unsafe fn new2(rect: *const Rect) -> Self {
        let mut __bindgen_tmp = MaybeUninit::uninit();
        Region_Region2(__bindgen_tmp.as_mut_ptr(), rect);
        __bindgen_tmp.assume_init()
    }
    #[inline]
    pub unsafe fn new3(path: *const GraphicsPath) -> Self {
        let mut __bindgen_tmp = MaybeUninit::uninit();
        Region_Region3(__bindgen_tmp.as_mut_ptr(), path);
        __bindgen_tmp.assume_init()
    }
    #[inline]
    pub unsafe fn new4(regionData: *const BYTE, size: INT) -> Self {
        let mut __bindgen_tmp = MaybeUninit::uninit();
        Region_Region4(__bindgen_tmp.as_mut_ptr(), regionData, size);
        __bindgen_tmp.assume_init()
    }
    #[inline]
    pub unsafe fn new5(hRgn: HRGN) -> Self {
        let mut __bindgen_tmp = MaybeUninit::uninit();
        Region_Region5(__bindgen_tmp.as_mut_ptr(), hRgn);
        __bindgen_tmp.assume_init()
    }
    #[inline]
    pub unsafe fn new6(nativeRegion: *mut GpRegion) -> Self {
        let mut __bindgen_tmp = MaybeUninit::uninit();
        Region_Region6(__bindgen_tmp.as_mut_ptr(), nativeRegion);
        __bindgen_tmp.assume_init()
    }
    #[inline]
    pub unsafe fn destruct(&mut self) {
        Region_Region_destructor(self)
    }
}
#[repr(C)]
#[derive(Debug)]
pub struct FontFamily {
    pub nativeFamily: *mut GpFontFamily,
    pub lastResult: Status,
}
extern "C" {
    #[link_name = "\u{1}?GenericSansSerif@FontFamily@Gdiplus@@SAPEBV12@XZ"]
    pub fn FontFamily_GenericSansSerif() -> *const FontFamily;
}
extern "C" {
    #[link_name = "\u{1}?GenericSerif@FontFamily@Gdiplus@@SAPEBV12@XZ"]
    pub fn FontFamily_GenericSerif() -> *const FontFamily;
}
extern "C" {
    #[link_name = "\u{1}?GenericMonospace@FontFamily@Gdiplus@@SAPEBV12@XZ"]
    pub fn FontFamily_GenericMonospace() -> *const FontFamily;
}
extern "C" {
    #[link_name = "\u{1}?GetFamilyName@FontFamily@Gdiplus@@QEBA?AW4Status@2@PEA_WG@Z"]
    pub fn FontFamily_GetFamilyName(
        this: *const FontFamily,
        name: LPWSTR,
        language: LANGID,
    ) -> Status;
}
extern "C" {
    #[link_name = "\u{1}?Clone@FontFamily@Gdiplus@@QEBAPEAV12@XZ"]
    pub fn FontFamily_Clone(this: *const FontFamily) -> *mut FontFamily;
}
extern "C" {
    #[link_name = "\u{1}?IsStyleAvailable@FontFamily@Gdiplus@@QEBAHH@Z"]
    pub fn FontFamily_IsStyleAvailable(this: *const FontFamily, style: INT) -> BOOL;
}
extern "C" {
    #[link_name = "\u{1}?GetEmHeight@FontFamily@Gdiplus@@QEBAGH@Z"]
    pub fn FontFamily_GetEmHeight(this: *const FontFamily, style: INT) -> UINT16;
}
extern "C" {
    #[link_name = "\u{1}?GetCellAscent@FontFamily@Gdiplus@@QEBAGH@Z"]
    pub fn FontFamily_GetCellAscent(this: *const FontFamily, style: INT) -> UINT16;
}
extern "C" {
    #[link_name = "\u{1}?GetCellDescent@FontFamily@Gdiplus@@QEBAGH@Z"]
    pub fn FontFamily_GetCellDescent(this: *const FontFamily, style: INT) -> UINT16;
}
extern "C" {
    #[link_name = "\u{1}?GetLineSpacing@FontFamily@Gdiplus@@QEBAGH@Z"]
    pub fn FontFamily_GetLineSpacing(this: *const FontFamily, style: INT) -> UINT16;
}
extern "C" {
    #[link_name = "\u{1}?GetLastStatus@FontFamily@Gdiplus@@QEBA?AW4Status@2@XZ"]
    pub fn FontFamily_GetLastStatus(this: *const FontFamily) -> Status;
}
extern "C" {
    #[link_name = "\u{1}?SetStatus@FontFamily@Gdiplus@@IEBA?AW4Status@2@W432@@Z"]
    pub fn FontFamily_SetStatus(this: *const FontFamily, status: Status) -> Status;
}
extern "C" {
    #[link_name = "\u{1}??0FontFamily@Gdiplus@@QEAA@XZ"]
    pub fn FontFamily_FontFamily(this: *mut FontFamily);
}
extern "C" {
    #[link_name = "\u{1}??0FontFamily@Gdiplus@@QEAA@PEB_WPEBVFontCollection@1@@Z"]
    pub fn FontFamily_FontFamily1(
        this: *mut FontFamily,
        name: *const WCHAR,
        fontCollection: *const FontCollection,
    );
}
extern "C" {
    #[link_name = "\u{1}??0FontFamily@Gdiplus@@IEAA@PEAVGpFontFamily@1@W4Status@1@@Z"]
    pub fn FontFamily_FontFamily2(
        this: *mut FontFamily,
        nativeFamily: *mut GpFontFamily,
        status: Status,
    );
}
extern "C" {
    #[link_name = "\u{1}??_DFontFamily@Gdiplus@@QEAAXXZ"]
    pub fn FontFamily_FontFamily_destructor(this: *mut FontFamily);
}
impl FontFamily {
    #[inline]
    pub unsafe fn GenericSansSerif() -> *const FontFamily {
        FontFamily_GenericSansSerif()
    }
    #[inline]
    pub unsafe fn GenericSerif() -> *const FontFamily {
        FontFamily_GenericSerif()
    }
    #[inline]
    pub unsafe fn GenericMonospace() -> *const FontFamily {
        FontFamily_GenericMonospace()
    }
    #[inline]
    pub unsafe fn GetFamilyName(&self, name: LPWSTR, language: LANGID) -> Status {
        FontFamily_GetFamilyName(self, name, language)
    }
    #[inline]
    pub unsafe fn Clone(&self) -> *mut FontFamily {
        FontFamily_Clone(self)
    }
    #[inline]
    pub unsafe fn IsStyleAvailable(&self, style: INT) -> BOOL {
        FontFamily_IsStyleAvailable(self, style)
    }
    #[inline]
    pub unsafe fn GetEmHeight(&self, style: INT) -> UINT16 {
        FontFamily_GetEmHeight(self, style)
    }
    #[inline]
    pub unsafe fn GetCellAscent(&self, style: INT) -> UINT16 {
        FontFamily_GetCellAscent(self, style)
    }
    #[inline]
    pub unsafe fn GetCellDescent(&self, style: INT) -> UINT16 {
        FontFamily_GetCellDescent(self, style)
    }
    #[inline]
    pub unsafe fn GetLineSpacing(&self, style: INT) -> UINT16 {
        FontFamily_GetLineSpacing(self, style)
    }
    #[inline]
    pub unsafe fn GetLastStatus(&self) -> Status {
        FontFamily_GetLastStatus(self)
    }
    #[inline]
    pub unsafe fn SetStatus(&self, status: Status) -> Status {
        FontFamily_SetStatus(self, status)
    }
    #[inline]
    pub unsafe fn new() -> Self {
        let mut __bindgen_tmp = MaybeUninit::uninit();
        FontFamily_FontFamily(__bindgen_tmp.as_mut_ptr());
        __bindgen_tmp.assume_init()
    }
    #[inline]
    pub unsafe fn new1(name: *const WCHAR, fontCollection: *const FontCollection) -> Self {
        let mut __bindgen_tmp = MaybeUninit::uninit();
        FontFamily_FontFamily1(__bindgen_tmp.as_mut_ptr(), name, fontCollection);
        __bindgen_tmp.assume_init()
    }
    #[inline]
    pub unsafe fn new2(nativeFamily: *mut GpFontFamily, status: Status) -> Self {
        let mut __bindgen_tmp = MaybeUninit::uninit();
        FontFamily_FontFamily2(__bindgen_tmp.as_mut_ptr(), nativeFamily, status);
        __bindgen_tmp.assume_init()
    }
    #[inline]
    pub unsafe fn destruct(&mut self) {
        FontFamily_FontFamily_destructor(self)
    }
}
#[repr(C)]
#[derive(Debug)]
pub struct Font {
    pub nativeFont: *mut GpFont,
    pub lastResult: Status,
}
extern "C" {
    #[link_name = "\u{1}?GetLogFontA@Font@Gdiplus@@QEBA?AW4Status@2@PEBVGraphics@2@PEAUtagLOGFONTA@@@Z"]
    pub fn Font_GetLogFontA(
        this: *const Font,
        g: *const Graphics,
        logfontA: *mut LOGFONTA,
    ) -> Status;
}
extern "C" {
    #[link_name = "\u{1}?GetLogFontW@Font@Gdiplus@@QEBA?AW4Status@2@PEBVGraphics@2@PEAUtagLOGFONTW@@@Z"]
    pub fn Font_GetLogFontW(
        this: *const Font,
        g: *const Graphics,
        logfontW: *mut LOGFONTW,
    ) -> Status;
}
extern "C" {
    #[link_name = "\u{1}?Clone@Font@Gdiplus@@QEBAPEAV12@XZ"]
    pub fn Font_Clone(this: *const Font) -> *mut Font;
}
extern "C" {
    #[link_name = "\u{1}?IsAvailable@Font@Gdiplus@@QEBAHXZ"]
    pub fn Font_IsAvailable(this: *const Font) -> BOOL;
}
extern "C" {
    #[link_name = "\u{1}?GetStyle@Font@Gdiplus@@QEBAHXZ"]
    pub fn Font_GetStyle(this: *const Font) -> INT;
}
extern "C" {
    #[link_name = "\u{1}?GetSize@Font@Gdiplus@@QEBAMXZ"]
    pub fn Font_GetSize(this: *const Font) -> REAL;
}
extern "C" {
    #[link_name = "\u{1}?GetUnit@Font@Gdiplus@@QEBA?AW4Unit@2@XZ"]
    pub fn Font_GetUnit(this: *const Font) -> Unit;
}
extern "C" {
    #[link_name = "\u{1}?GetLastStatus@Font@Gdiplus@@QEBA?AW4Status@2@XZ"]
    pub fn Font_GetLastStatus(this: *const Font) -> Status;
}
extern "C" {
    #[link_name = "\u{1}?GetHeight@Font@Gdiplus@@QEBAMPEBVGraphics@2@@Z"]
    pub fn Font_GetHeight(this: *const Font, graphics: *const Graphics) -> REAL;
}
extern "C" {
    #[link_name = "\u{1}?GetHeight@Font@Gdiplus@@QEBAMM@Z"]
    pub fn Font_GetHeight1(this: *const Font, dpi: REAL) -> REAL;
}
extern "C" {
    #[link_name = "\u{1}?GetFamily@Font@Gdiplus@@QEBA?AW4Status@2@PEAVFontFamily@2@@Z"]
    pub fn Font_GetFamily(this: *const Font, family: *mut FontFamily) -> Status;
}
extern "C" {
    #[link_name = "\u{1}?SetNativeFont@Font@Gdiplus@@IEAAXPEAVGpFont@2@@Z"]
    pub fn Font_SetNativeFont(this: *mut Font, Font: *mut GpFont);
}
extern "C" {
    #[link_name = "\u{1}?SetStatus@Font@Gdiplus@@IEBA?AW4Status@2@W432@@Z"]
    pub fn Font_SetStatus(this: *const Font, status: Status) -> Status;
}
extern "C" {
    #[link_name = "\u{1}??0Font@Gdiplus@@QEAA@PEAUHDC__@@@Z"]
    pub fn Font_Font(this: *mut Font, hdc: HDC);
}
extern "C" {
    #[link_name = "\u{1}??0Font@Gdiplus@@QEAA@PEAUHDC__@@PEBUtagLOGFONTA@@@Z"]
    pub fn Font_Font1(this: *mut Font, hdc: HDC, logfont: *const LOGFONTA);
}
extern "C" {
    #[link_name = "\u{1}??0Font@Gdiplus@@QEAA@PEAUHDC__@@PEBUtagLOGFONTW@@@Z"]
    pub fn Font_Font2(this: *mut Font, hdc: HDC, logfont: *const LOGFONTW);
}
extern "C" {
    #[link_name = "\u{1}??0Font@Gdiplus@@QEAA@PEAUHDC__@@QEAUHFONT__@@@Z"]
    pub fn Font_Font3(this: *mut Font, hdc: HDC, hfont: HFONT);
}
extern "C" {
    #[link_name = "\u{1}??0Font@Gdiplus@@QEAA@PEBVFontFamily@1@MHW4Unit@1@@Z"]
    pub fn Font_Font4(
        this: *mut Font,
        family: *const FontFamily,
        emSize: REAL,
        style: INT,
        unit: Unit,
    );
}
extern "C" {
    #[link_name = "\u{1}??0Font@Gdiplus@@QEAA@PEB_WMHW4Unit@1@PEBVFontCollection@1@@Z"]
    pub fn Font_Font5(
        this: *mut Font,
        familyName: *const WCHAR,
        emSize: REAL,
        style: INT,
        unit: Unit,
        fontCollection: *const FontCollection,
    );
}
extern "C" {
    #[link_name = "\u{1}??0Font@Gdiplus@@IEAA@PEAVGpFont@1@W4Status@1@@Z"]
    pub fn Font_Font6(this: *mut Font, font: *mut GpFont, status: Status);
}
extern "C" {
    #[link_name = "\u{1}??_DFont@Gdiplus@@QEAAXXZ"]
    pub fn Font_Font_destructor(this: *mut Font);
}
impl Font {
    #[inline]
    pub unsafe fn GetLogFontA(&self, g: *const Graphics, logfontA: *mut LOGFONTA) -> Status {
        Font_GetLogFontA(self, g, logfontA)
    }
    #[inline]
    pub unsafe fn GetLogFontW(&self, g: *const Graphics, logfontW: *mut LOGFONTW) -> Status {
        Font_GetLogFontW(self, g, logfontW)
    }
    #[inline]
    pub unsafe fn Clone(&self) -> *mut Font {
        Font_Clone(self)
    }
    #[inline]
    pub unsafe fn IsAvailable(&self) -> BOOL {
        Font_IsAvailable(self)
    }
    #[inline]
    pub unsafe fn GetStyle(&self) -> INT {
        Font_GetStyle(self)
    }
    #[inline]
    pub unsafe fn GetSize(&self) -> REAL {
        Font_GetSize(self)
    }
    #[inline]
    pub unsafe fn GetUnit(&self) -> Unit {
        Font_GetUnit(self)
    }
    #[inline]
    pub unsafe fn GetLastStatus(&self) -> Status {
        Font_GetLastStatus(self)
    }
    #[inline]
    pub unsafe fn GetHeight(&self, graphics: *const Graphics) -> REAL {
        Font_GetHeight(self, graphics)
    }
    #[inline]
    pub unsafe fn GetHeight1(&self, dpi: REAL) -> REAL {
        Font_GetHeight1(self, dpi)
    }
    #[inline]
    pub unsafe fn GetFamily(&self, family: *mut FontFamily) -> Status {
        Font_GetFamily(self, family)
    }
    #[inline]
    pub unsafe fn SetNativeFont(&mut self, Font: *mut GpFont) {
        Font_SetNativeFont(self, Font)
    }
    #[inline]
    pub unsafe fn SetStatus(&self, status: Status) -> Status {
        Font_SetStatus(self, status)
    }
    #[inline]
    pub unsafe fn new(hdc: HDC) -> Self {
        let mut __bindgen_tmp = MaybeUninit::uninit();
        Font_Font(__bindgen_tmp.as_mut_ptr(), hdc);
        __bindgen_tmp.assume_init()
    }
    #[inline]
    pub unsafe fn new1(hdc: HDC, logfont: *const LOGFONTA) -> Self {
        let mut __bindgen_tmp = MaybeUninit::uninit();
        Font_Font1(__bindgen_tmp.as_mut_ptr(), hdc, logfont);
        __bindgen_tmp.assume_init()
    }
    #[inline]
    pub unsafe fn new2(hdc: HDC, logfont: *const LOGFONTW) -> Self {
        let mut __bindgen_tmp = MaybeUninit::uninit();
        Font_Font2(__bindgen_tmp.as_mut_ptr(), hdc, logfont);
        __bindgen_tmp.assume_init()
    }
    #[inline]
    pub unsafe fn new3(hdc: HDC, hfont: HFONT) -> Self {
        let mut __bindgen_tmp = MaybeUninit::uninit();
        Font_Font3(__bindgen_tmp.as_mut_ptr(), hdc, hfont);
        __bindgen_tmp.assume_init()
    }
    #[inline]
    pub unsafe fn new4(family: *const FontFamily, emSize: REAL, style: INT, unit: Unit) -> Self {
        let mut __bindgen_tmp = MaybeUninit::uninit();
        Font_Font4(__bindgen_tmp.as_mut_ptr(), family, emSize, style, unit);
        __bindgen_tmp.assume_init()
    }
    #[inline]
    pub unsafe fn new5(
        familyName: *const WCHAR,
        emSize: REAL,
        style: INT,
        unit: Unit,
        fontCollection: *const FontCollection,
    ) -> Self {
        let mut __bindgen_tmp = MaybeUninit::uninit();
        Font_Font5(
            __bindgen_tmp.as_mut_ptr(),
            familyName,
            emSize,
            style,
            unit,
            fontCollection,
        );
        __bindgen_tmp.assume_init()
    }
    #[inline]
    pub unsafe fn new6(font: *mut GpFont, status: Status) -> Self {
        let mut __bindgen_tmp = MaybeUninit::uninit();
        Font_Font6(__bindgen_tmp.as_mut_ptr(), font, status);
        __bindgen_tmp.assume_init()
    }
    #[inline]
    pub unsafe fn destruct(&mut self) {
        Font_Font_destructor(self)
    }
}
#[repr(C)]
pub struct FontCollection__bindgen_vtable(c_void);
#[repr(C)]
#[derive(Debug)]
pub struct FontCollection {
    pub vtable_: *const FontCollection__bindgen_vtable,
    pub nativeFontCollection: *mut GpFontCollection,
    pub lastResult: Status,
}
extern "C" {
    #[link_name = "\u{1}?GetFamilyCount@FontCollection@Gdiplus@@QEBAHXZ"]
    pub fn FontCollection_GetFamilyCount(this: *const FontCollection) -> INT;
}
extern "C" {
    #[link_name = "\u{1}?GetFamilies@FontCollection@Gdiplus@@QEBA?AW4Status@2@HPEAVFontFamily@2@PEAH@Z"]
    pub fn FontCollection_GetFamilies(
        this: *const FontCollection,
        numSought: INT,
        gpfamilies: *mut FontFamily,
        numFound: *mut INT,
    ) -> Status;
}
extern "C" {
    #[link_name = "\u{1}?GetLastStatus@FontCollection@Gdiplus@@QEBA?AW4Status@2@XZ"]
    pub fn FontCollection_GetLastStatus(this: *const FontCollection) -> Status;
}
extern "C" {
    #[link_name = "\u{1}?SetStatus@FontCollection@Gdiplus@@IEBA?AW4Status@2@W432@@Z"]
    pub fn FontCollection_SetStatus(this: *const FontCollection, status: Status) -> Status;
}
extern "C" {
    #[link_name = "\u{1}??0FontCollection@Gdiplus@@QEAA@XZ"]
    pub fn FontCollection_FontCollection(this: *mut FontCollection);
}
impl FontCollection {
    #[inline]
    pub unsafe fn GetFamilyCount(&self) -> INT {
        FontCollection_GetFamilyCount(self)
    }
    #[inline]
    pub unsafe fn GetFamilies(
        &self,
        numSought: INT,
        gpfamilies: *mut FontFamily,
        numFound: *mut INT,
    ) -> Status {
        FontCollection_GetFamilies(self, numSought, gpfamilies, numFound)
    }
    #[inline]
    pub unsafe fn GetLastStatus(&self) -> Status {
        FontCollection_GetLastStatus(self)
    }
    #[inline]
    pub unsafe fn SetStatus(&self, status: Status) -> Status {
        FontCollection_SetStatus(self, status)
    }
    #[inline]
    pub unsafe fn new() -> Self {
        let mut __bindgen_tmp = MaybeUninit::uninit();
        FontCollection_FontCollection(__bindgen_tmp.as_mut_ptr());
        __bindgen_tmp.assume_init()
    }
}
extern "C" {
    #[link_name = "\u{1}??_DFontCollection@Gdiplus@@QEAAXXZ"]
    pub fn FontCollection_FontCollection_destructor(this: *mut FontCollection);
}
#[repr(C)]
#[derive(Debug)]
pub struct InstalledFontCollection {
    pub _base: FontCollection,
}
extern "C" {
    #[link_name = "\u{1}?SetStatus@InstalledFontCollection@Gdiplus@@IEBA?AW4Status@2@W432@@Z"]
    pub fn InstalledFontCollection_SetStatus(
        this: *const InstalledFontCollection,
        status: Status,
    ) -> Status;
}
extern "C" {
    #[link_name = "\u{1}??0InstalledFontCollection@Gdiplus@@QEAA@XZ"]
    pub fn InstalledFontCollection_InstalledFontCollection(this: *mut InstalledFontCollection);
}
impl InstalledFontCollection {
    #[inline]
    pub unsafe fn SetStatus(&self, status: Status) -> Status {
        InstalledFontCollection_SetStatus(self, status)
    }
    #[inline]
    pub unsafe fn new() -> Self {
        let mut __bindgen_tmp = MaybeUninit::uninit();
        InstalledFontCollection_InstalledFontCollection(__bindgen_tmp.as_mut_ptr());
        __bindgen_tmp.assume_init()
    }
}
extern "C" {
    #[link_name = "\u{1}??_DInstalledFontCollection@Gdiplus@@QEAAXXZ"]
    pub fn InstalledFontCollection_InstalledFontCollection_destructor(
        this: *mut InstalledFontCollection,
    );
}
#[repr(C)]
#[derive(Debug)]
pub struct PrivateFontCollection {
    pub _base: FontCollection,
}
extern "C" {
    #[link_name = "\u{1}?AddFontFile@PrivateFontCollection@Gdiplus@@QEAA?AW4Status@2@PEB_W@Z"]
    pub fn PrivateFontCollection_AddFontFile(
        this: *mut PrivateFontCollection,
        filename: *const WCHAR,
    ) -> Status;
}
extern "C" {
    #[link_name = "\u{1}?AddMemoryFont@PrivateFontCollection@Gdiplus@@QEAA?AW4Status@2@PEBXH@Z"]
    pub fn PrivateFontCollection_AddMemoryFont(
        this: *mut PrivateFontCollection,
        memory: *const c_void,
        length: INT,
    ) -> Status;
}
extern "C" {
    #[link_name = "\u{1}??0PrivateFontCollection@Gdiplus@@QEAA@XZ"]
    pub fn PrivateFontCollection_PrivateFontCollection(this: *mut PrivateFontCollection);
}
impl PrivateFontCollection {
    #[inline]
    pub unsafe fn AddFontFile(&mut self, filename: *const WCHAR) -> Status {
        PrivateFontCollection_AddFontFile(self, filename)
    }
    #[inline]
    pub unsafe fn AddMemoryFont(&mut self, memory: *const c_void, length: INT) -> Status {
        PrivateFontCollection_AddMemoryFont(self, memory, length)
    }
    #[inline]
    pub unsafe fn new() -> Self {
        let mut __bindgen_tmp = MaybeUninit::uninit();
        PrivateFontCollection_PrivateFontCollection(__bindgen_tmp.as_mut_ptr());
        __bindgen_tmp.assume_init()
    }
}
extern "C" {
    #[link_name = "\u{1}??_DPrivateFontCollection@Gdiplus@@QEAAXXZ"]
    pub fn PrivateFontCollection_PrivateFontCollection_destructor(this: *mut PrivateFontCollection);
}
#[repr(C)]
pub struct Image__bindgen_vtable(c_void);
#[repr(C)]
#[derive(Debug)]
pub struct Image {
    pub vtable_: *const Image__bindgen_vtable,
    pub nativeImage: *mut GpImage,
    pub lastResult: Status,
    pub loadStatus: Status,
}
extern "C" {
    #[link_name = "\u{1}?FromFile@Image@Gdiplus@@SAPEAV12@PEB_WH@Z"]
    pub fn Image_FromFile(filename: *const WCHAR, useEmbeddedColorManagement: BOOL) -> *mut Image;
}
extern "C" {
    #[link_name = "\u{1}?FromStream@Image@Gdiplus@@SAPEAV12@PEAUIStream@@H@Z"]
    pub fn Image_FromStream(stream: *mut IStream, useEmbeddedColorManagement: BOOL) -> *mut Image;
}
extern "C" {
    #[link_name = "\u{1}?Save@Image@Gdiplus@@QEAA?AW4Status@2@PEB_WPEBU_GUID@@PEBVEncoderParameters@2@@Z"]
    pub fn Image_Save(
        this: *mut Image,
        filename: *const WCHAR,
        clsidEncoder: *const CLSID,
        encoderParams: *const EncoderParameters,
    ) -> Status;
}
extern "C" {
    #[link_name = "\u{1}?Save@Image@Gdiplus@@QEAA?AW4Status@2@PEAUIStream@@PEBU_GUID@@PEBVEncoderParameters@2@@Z"]
    pub fn Image_Save1(
        this: *mut Image,
        stream: *mut IStream,
        clsidEncoder: *const CLSID,
        encoderParams: *const EncoderParameters,
    ) -> Status;
}
extern "C" {
    #[link_name = "\u{1}?SaveAdd@Image@Gdiplus@@QEAA?AW4Status@2@PEBVEncoderParameters@2@@Z"]
    pub fn Image_SaveAdd(this: *mut Image, encoderParams: *const EncoderParameters) -> Status;
}
extern "C" {
    #[link_name = "\u{1}?SaveAdd@Image@Gdiplus@@QEAA?AW4Status@2@PEAV12@PEBVEncoderParameters@2@@Z"]
    pub fn Image_SaveAdd1(
        this: *mut Image,
        newImage: *mut Image,
        encoderParams: *const EncoderParameters,
    ) -> Status;
}
extern "C" {
    #[link_name = "\u{1}?GetType@Image@Gdiplus@@QEBA?AW4ImageType@2@XZ"]
    pub fn Image_GetType(this: *const Image) -> ImageType;
}
extern "C" {
    #[link_name = "\u{1}?GetPhysicalDimension@Image@Gdiplus@@QEAA?AW4Status@2@PEAVSizeF@2@@Z"]
    pub fn Image_GetPhysicalDimension(this: *mut Image, size: *mut SizeF) -> Status;
}
extern "C" {
    #[link_name = "\u{1}?GetBounds@Image@Gdiplus@@QEAA?AW4Status@2@PEAVRectF@2@PEAW4Unit@2@@Z"]
    pub fn Image_GetBounds(this: *mut Image, srcRect: *mut RectF, srcUnit: *mut Unit) -> Status;
}
extern "C" {
    #[link_name = "\u{1}?GetWidth@Image@Gdiplus@@QEAAIXZ"]
    pub fn Image_GetWidth(this: *mut Image) -> UINT;
}
extern "C" {
    #[link_name = "\u{1}?GetHeight@Image@Gdiplus@@QEAAIXZ"]
    pub fn Image_GetHeight(this: *mut Image) -> UINT;
}
extern "C" {
    #[link_name = "\u{1}?GetHorizontalResolution@Image@Gdiplus@@QEAAMXZ"]
    pub fn Image_GetHorizontalResolution(this: *mut Image) -> REAL;
}
extern "C" {
    #[link_name = "\u{1}?GetVerticalResolution@Image@Gdiplus@@QEAAMXZ"]
    pub fn Image_GetVerticalResolution(this: *mut Image) -> REAL;
}
extern "C" {
    #[link_name = "\u{1}?GetFlags@Image@Gdiplus@@QEAAIXZ"]
    pub fn Image_GetFlags(this: *mut Image) -> UINT;
}
extern "C" {
    #[link_name = "\u{1}?GetRawFormat@Image@Gdiplus@@QEAA?AW4Status@2@PEAU_GUID@@@Z"]
    pub fn Image_GetRawFormat(this: *mut Image, format: *mut GUID) -> Status;
}
extern "C" {
    #[link_name = "\u{1}?GetPixelFormat@Image@Gdiplus@@QEAAHXZ"]
    pub fn Image_GetPixelFormat(this: *mut Image) -> PixelFormat;
}
extern "C" {
    #[link_name = "\u{1}?GetPaletteSize@Image@Gdiplus@@QEAAHXZ"]
    pub fn Image_GetPaletteSize(this: *mut Image) -> INT;
}
extern "C" {
    #[link_name = "\u{1}?GetPalette@Image@Gdiplus@@QEAA?AW4Status@2@PEAUColorPalette@2@H@Z"]
    pub fn Image_GetPalette(this: *mut Image, palette: *mut ColorPalette, size: INT) -> Status;
}
extern "C" {
    #[link_name = "\u{1}?SetPalette@Image@Gdiplus@@QEAA?AW4Status@2@PEBUColorPalette@2@@Z"]
    pub fn Image_SetPalette(this: *mut Image, palette: *const ColorPalette) -> Status;
}
extern "C" {
    #[link_name = "\u{1}?GetThumbnailImage@Image@Gdiplus@@QEAAPEAV12@IIP6AHPEAX@Z0@Z"]
    pub fn Image_GetThumbnailImage(
        this: *mut Image,
        thumbWidth: UINT,
        thumbHeight: UINT,
        callback: GetThumbnailImageAbort,
        callbackData: *mut c_void,
    ) -> *mut Image;
}
extern "C" {
    #[link_name = "\u{1}?GetFrameDimensionsCount@Image@Gdiplus@@QEAAIXZ"]
    pub fn Image_GetFrameDimensionsCount(this: *mut Image) -> UINT;
}
extern "C" {
    #[link_name = "\u{1}?GetFrameDimensionsList@Image@Gdiplus@@QEAA?AW4Status@2@PEAU_GUID@@I@Z"]
    pub fn Image_GetFrameDimensionsList(
        this: *mut Image,
        dimensionIDs: *mut GUID,
        count: UINT,
    ) -> Status;
}
extern "C" {
    #[link_name = "\u{1}?GetFrameCount@Image@Gdiplus@@QEAAIPEBU_GUID@@@Z"]
    pub fn Image_GetFrameCount(this: *mut Image, dimensionID: *const GUID) -> UINT;
}
extern "C" {
    #[link_name = "\u{1}?SelectActiveFrame@Image@Gdiplus@@QEAA?AW4Status@2@PEBU_GUID@@I@Z"]
    pub fn Image_SelectActiveFrame(
        this: *mut Image,
        dimensionID: *const GUID,
        frameIndex: UINT,
    ) -> Status;
}
extern "C" {
    #[link_name = "\u{1}?RotateFlip@Image@Gdiplus@@QEAA?AW4Status@2@W4RotateFlipType@2@@Z"]
    pub fn Image_RotateFlip(this: *mut Image, rotateFlipType: RotateFlipType) -> Status;
}
extern "C" {
    #[link_name = "\u{1}?GetPropertyCount@Image@Gdiplus@@QEAAIXZ"]
    pub fn Image_GetPropertyCount(this: *mut Image) -> UINT;
}
extern "C" {
    #[link_name = "\u{1}?GetPropertyIdList@Image@Gdiplus@@QEAA?AW4Status@2@IPEAK@Z"]
    pub fn Image_GetPropertyIdList(
        this: *mut Image,
        numOfProperty: UINT,
        list: *mut PROPID,
    ) -> Status;
}
extern "C" {
    #[link_name = "\u{1}?GetPropertyItemSize@Image@Gdiplus@@QEAAIK@Z"]
    pub fn Image_GetPropertyItemSize(this: *mut Image, propId: PROPID) -> UINT;
}
extern "C" {
    #[link_name = "\u{1}?GetPropertyItem@Image@Gdiplus@@QEAA?AW4Status@2@KIPEAVPropertyItem@2@@Z"]
    pub fn Image_GetPropertyItem(
        this: *mut Image,
        propId: PROPID,
        propSize: UINT,
        buffer: *mut PropertyItem,
    ) -> Status;
}
extern "C" {
    #[link_name = "\u{1}?GetPropertySize@Image@Gdiplus@@QEAA?AW4Status@2@PEAI0@Z"]
    pub fn Image_GetPropertySize(
        this: *mut Image,
        totalBufferSize: *mut UINT,
        numProperties: *mut UINT,
    ) -> Status;
}
extern "C" {
    #[link_name = "\u{1}?GetAllPropertyItems@Image@Gdiplus@@QEAA?AW4Status@2@IIPEAVPropertyItem@2@@Z"]
    pub fn Image_GetAllPropertyItems(
        this: *mut Image,
        totalBufferSize: UINT,
        numProperties: UINT,
        allItems: *mut PropertyItem,
    ) -> Status;
}
extern "C" {
    #[link_name = "\u{1}?RemovePropertyItem@Image@Gdiplus@@QEAA?AW4Status@2@K@Z"]
    pub fn Image_RemovePropertyItem(this: *mut Image, propId: PROPID) -> Status;
}
extern "C" {
    #[link_name = "\u{1}?SetPropertyItem@Image@Gdiplus@@QEAA?AW4Status@2@PEBVPropertyItem@2@@Z"]
    pub fn Image_SetPropertyItem(this: *mut Image, item: *const PropertyItem) -> Status;
}
extern "C" {
    #[link_name = "\u{1}?GetEncoderParameterListSize@Image@Gdiplus@@QEAAIPEBU_GUID@@@Z"]
    pub fn Image_GetEncoderParameterListSize(this: *mut Image, clsidEncoder: *const CLSID) -> UINT;
}
extern "C" {
    #[link_name = "\u{1}?GetEncoderParameterList@Image@Gdiplus@@QEAA?AW4Status@2@PEBU_GUID@@IPEAVEncoderParameters@2@@Z"]
    pub fn Image_GetEncoderParameterList(
        this: *mut Image,
        clsidEncoder: *const CLSID,
        size: UINT,
        buffer: *mut EncoderParameters,
    ) -> Status;
}
extern "C" {
    #[link_name = "\u{1}?GetLastStatus@Image@Gdiplus@@QEBA?AW4Status@2@XZ"]
    pub fn Image_GetLastStatus(this: *const Image) -> Status;
}
extern "C" {
    #[link_name = "\u{1}?SetNativeImage@Image@Gdiplus@@IEAAXPEAVGpImage@2@@Z"]
    pub fn Image_SetNativeImage(this: *mut Image, nativeImage: *mut GpImage);
}
extern "C" {
    #[link_name = "\u{1}??0Image@Gdiplus@@QEAA@PEB_WH@Z"]
    pub fn Image_Image(this: *mut Image, filename: *const WCHAR, useEmbeddedColorManagement: BOOL);
}
extern "C" {
    #[link_name = "\u{1}??0Image@Gdiplus@@QEAA@PEAUIStream@@H@Z"]
    pub fn Image_Image1(this: *mut Image, stream: *mut IStream, useEmbeddedColorManagement: BOOL);
}
extern "C" {
    #[link_name = "\u{1}??0Image@Gdiplus@@IEAA@PEAVGpImage@1@W4Status@1@@Z"]
    pub fn Image_Image2(this: *mut Image, nativeImage: *mut GpImage, status: Status);
}
impl Image {
    #[inline]
    pub unsafe fn FromFile(filename: *const WCHAR, useEmbeddedColorManagement: BOOL) -> *mut Image {
        Image_FromFile(filename, useEmbeddedColorManagement)
    }
    #[inline]
    pub unsafe fn FromStream(stream: *mut IStream, useEmbeddedColorManagement: BOOL) -> *mut Image {
        Image_FromStream(stream, useEmbeddedColorManagement)
    }
    #[inline]
    pub unsafe fn Save(
        &mut self,
        filename: *const WCHAR,
        clsidEncoder: *const CLSID,
        encoderParams: *const EncoderParameters,
    ) -> Status {
        Image_Save(self, filename, clsidEncoder, encoderParams)
    }
    #[inline]
    pub unsafe fn Save1(
        &mut self,
        stream: *mut IStream,
        clsidEncoder: *const CLSID,
        encoderParams: *const EncoderParameters,
    ) -> Status {
        Image_Save1(self, stream, clsidEncoder, encoderParams)
    }
    #[inline]
    pub unsafe fn SaveAdd(&mut self, encoderParams: *const EncoderParameters) -> Status {
        Image_SaveAdd(self, encoderParams)
    }
    #[inline]
    pub unsafe fn SaveAdd1(
        &mut self,
        newImage: *mut Image,
        encoderParams: *const EncoderParameters,
    ) -> Status {
        Image_SaveAdd1(self, newImage, encoderParams)
    }
    #[inline]
    pub unsafe fn GetType(&self) -> ImageType {
        Image_GetType(self)
    }
    #[inline]
    pub unsafe fn GetPhysicalDimension(&mut self, size: *mut SizeF) -> Status {
        Image_GetPhysicalDimension(self, size)
    }
    #[inline]
    pub unsafe fn GetBounds(&mut self, srcRect: *mut RectF, srcUnit: *mut Unit) -> Status {
        Image_GetBounds(self, srcRect, srcUnit)
    }
    #[inline]
    pub unsafe fn GetWidth(&mut self) -> UINT {
        Image_GetWidth(self)
    }
    #[inline]
    pub unsafe fn GetHeight(&mut self) -> UINT {
        Image_GetHeight(self)
    }
    #[inline]
    pub unsafe fn GetHorizontalResolution(&mut self) -> REAL {
        Image_GetHorizontalResolution(self)
    }
    #[inline]
    pub unsafe fn GetVerticalResolution(&mut self) -> REAL {
        Image_GetVerticalResolution(self)
    }
    #[inline]
    pub unsafe fn GetFlags(&mut self) -> UINT {
        Image_GetFlags(self)
    }
    #[inline]
    pub unsafe fn GetRawFormat(&mut self, format: *mut GUID) -> Status {
        Image_GetRawFormat(self, format)
    }
    #[inline]
    pub unsafe fn GetPixelFormat(&mut self) -> PixelFormat {
        Image_GetPixelFormat(self)
    }
    #[inline]
    pub unsafe fn GetPaletteSize(&mut self) -> INT {
        Image_GetPaletteSize(self)
    }
    #[inline]
    pub unsafe fn GetPalette(&mut self, palette: *mut ColorPalette, size: INT) -> Status {
        Image_GetPalette(self, palette, size)
    }
    #[inline]
    pub unsafe fn SetPalette(&mut self, palette: *const ColorPalette) -> Status {
        Image_SetPalette(self, palette)
    }
    #[inline]
    pub unsafe fn GetThumbnailImage(
        &mut self,
        thumbWidth: UINT,
        thumbHeight: UINT,
        callback: GetThumbnailImageAbort,
        callbackData: *mut c_void,
    ) -> *mut Image {
        Image_GetThumbnailImage(self, thumbWidth, thumbHeight, callback, callbackData)
    }
    #[inline]
    pub unsafe fn GetFrameDimensionsCount(&mut self) -> UINT {
        Image_GetFrameDimensionsCount(self)
    }
    #[inline]
    pub unsafe fn GetFrameDimensionsList(
        &mut self,
        dimensionIDs: *mut GUID,
        count: UINT,
    ) -> Status {
        Image_GetFrameDimensionsList(self, dimensionIDs, count)
    }
    #[inline]
    pub unsafe fn GetFrameCount(&mut self, dimensionID: *const GUID) -> UINT {
        Image_GetFrameCount(self, dimensionID)
    }
    #[inline]
    pub unsafe fn SelectActiveFrame(
        &mut self,
        dimensionID: *const GUID,
        frameIndex: UINT,
    ) -> Status {
        Image_SelectActiveFrame(self, dimensionID, frameIndex)
    }
    #[inline]
    pub unsafe fn RotateFlip(&mut self, rotateFlipType: RotateFlipType) -> Status {
        Image_RotateFlip(self, rotateFlipType)
    }
    #[inline]
    pub unsafe fn GetPropertyCount(&mut self) -> UINT {
        Image_GetPropertyCount(self)
    }
    #[inline]
    pub unsafe fn GetPropertyIdList(&mut self, numOfProperty: UINT, list: *mut PROPID) -> Status {
        Image_GetPropertyIdList(self, numOfProperty, list)
    }
    #[inline]
    pub unsafe fn GetPropertyItemSize(&mut self, propId: PROPID) -> UINT {
        Image_GetPropertyItemSize(self, propId)
    }
    #[inline]
    pub unsafe fn GetPropertyItem(
        &mut self,
        propId: PROPID,
        propSize: UINT,
        buffer: *mut PropertyItem,
    ) -> Status {
        Image_GetPropertyItem(self, propId, propSize, buffer)
    }
    #[inline]
    pub unsafe fn GetPropertySize(
        &mut self,
        totalBufferSize: *mut UINT,
        numProperties: *mut UINT,
    ) -> Status {
        Image_GetPropertySize(self, totalBufferSize, numProperties)
    }
    #[inline]
    pub unsafe fn GetAllPropertyItems(
        &mut self,
        totalBufferSize: UINT,
        numProperties: UINT,
        allItems: *mut PropertyItem,
    ) -> Status {
        Image_GetAllPropertyItems(self, totalBufferSize, numProperties, allItems)
    }
    #[inline]
    pub unsafe fn RemovePropertyItem(&mut self, propId: PROPID) -> Status {
        Image_RemovePropertyItem(self, propId)
    }
    #[inline]
    pub unsafe fn SetPropertyItem(&mut self, item: *const PropertyItem) -> Status {
        Image_SetPropertyItem(self, item)
    }
    #[inline]
    pub unsafe fn GetEncoderParameterListSize(&mut self, clsidEncoder: *const CLSID) -> UINT {
        Image_GetEncoderParameterListSize(self, clsidEncoder)
    }
    #[inline]
    pub unsafe fn GetEncoderParameterList(
        &mut self,
        clsidEncoder: *const CLSID,
        size: UINT,
        buffer: *mut EncoderParameters,
    ) -> Status {
        Image_GetEncoderParameterList(self, clsidEncoder, size, buffer)
    }
    #[inline]
    pub unsafe fn GetLastStatus(&self) -> Status {
        Image_GetLastStatus(self)
    }
    #[inline]
    pub unsafe fn SetNativeImage(&mut self, nativeImage: *mut GpImage) {
        Image_SetNativeImage(self, nativeImage)
    }
    #[inline]
    pub unsafe fn new(filename: *const WCHAR, useEmbeddedColorManagement: BOOL) -> Self {
        let mut __bindgen_tmp = MaybeUninit::uninit();
        Image_Image(
            __bindgen_tmp.as_mut_ptr(),
            filename,
            useEmbeddedColorManagement,
        );
        __bindgen_tmp.assume_init()
    }
    #[inline]
    pub unsafe fn new1(stream: *mut IStream, useEmbeddedColorManagement: BOOL) -> Self {
        let mut __bindgen_tmp = MaybeUninit::uninit();
        Image_Image1(
            __bindgen_tmp.as_mut_ptr(),
            stream,
            useEmbeddedColorManagement,
        );
        __bindgen_tmp.assume_init()
    }
    #[inline]
    pub unsafe fn new2(nativeImage: *mut GpImage, status: Status) -> Self {
        let mut __bindgen_tmp = MaybeUninit::uninit();
        Image_Image2(__bindgen_tmp.as_mut_ptr(), nativeImage, status);
        __bindgen_tmp.assume_init()
    }
}
extern "C" {
    #[link_name = "\u{1}??_DImage@Gdiplus@@QEAAXXZ"]
    pub fn Image_Image_destructor(this: *mut Image);
}
extern "C" {
    #[link_name = "\u{1}?Clone@Image@Gdiplus@@UEAAPEAV12@XZ"]
    pub fn Image_Clone(this: *mut c_void) -> *mut Image;
}
#[repr(C)]
#[derive(Debug)]
pub struct Bitmap {
    pub _base: Image,
}
extern "C" {
    #[link_name = "\u{1}?FromFile@Bitmap@Gdiplus@@SAPEAV12@PEB_WH@Z"]
    pub fn Bitmap_FromFile(filename: *const WCHAR, useEmbeddedColorManagement: BOOL)
        -> *mut Bitmap;
}
extern "C" {
    #[link_name = "\u{1}?FromStream@Bitmap@Gdiplus@@SAPEAV12@PEAUIStream@@H@Z"]
    pub fn Bitmap_FromStream(stream: *mut IStream, useEmbeddedColorManagement: BOOL)
        -> *mut Bitmap;
}
extern "C" {
    #[link_name = "\u{1}?Clone@Bitmap@Gdiplus@@QEAAPEAV12@AEBVRect@2@H@Z"]
    pub fn Bitmap_Clone(this: *mut Bitmap, rect: *const Rect, format: PixelFormat) -> *mut Bitmap;
}
extern "C" {
    #[link_name = "\u{1}?Clone@Bitmap@Gdiplus@@QEAAPEAV12@HHHHH@Z"]
    pub fn Bitmap_Clone1(
        this: *mut Bitmap,
        x: INT,
        y: INT,
        width: INT,
        height: INT,
        format: PixelFormat,
    ) -> *mut Bitmap;
}
extern "C" {
    #[link_name = "\u{1}?Clone@Bitmap@Gdiplus@@QEAAPEAV12@AEBVRectF@2@H@Z"]
    pub fn Bitmap_Clone2(this: *mut Bitmap, rect: *const RectF, format: PixelFormat)
        -> *mut Bitmap;
}
extern "C" {
    #[link_name = "\u{1}?Clone@Bitmap@Gdiplus@@QEAAPEAV12@MMMMH@Z"]
    pub fn Bitmap_Clone3(
        this: *mut Bitmap,
        x: REAL,
        y: REAL,
        width: REAL,
        height: REAL,
        format: PixelFormat,
    ) -> *mut Bitmap;
}
extern "C" {
    #[link_name = "\u{1}?LockBits@Bitmap@Gdiplus@@QEAA?AW4Status@2@PEBVRect@2@IHPEAVBitmapData@2@@Z"]
    pub fn Bitmap_LockBits(
        this: *mut Bitmap,
        rect: *const Rect,
        flags: UINT,
        format: PixelFormat,
        lockedBitmapData: *mut BitmapData,
    ) -> Status;
}
extern "C" {
    #[link_name = "\u{1}?UnlockBits@Bitmap@Gdiplus@@QEAA?AW4Status@2@PEAVBitmapData@2@@Z"]
    pub fn Bitmap_UnlockBits(this: *mut Bitmap, lockedBitmapData: *mut BitmapData) -> Status;
}
extern "C" {
    #[link_name = "\u{1}?GetPixel@Bitmap@Gdiplus@@QEAA?AW4Status@2@HHPEAVColor@2@@Z"]
    pub fn Bitmap_GetPixel(this: *mut Bitmap, x: INT, y: INT, color: *mut Color) -> Status;
}
extern "C" {
    #[link_name = "\u{1}?SetPixel@Bitmap@Gdiplus@@QEAA?AW4Status@2@HHAEBVColor@2@@Z"]
    pub fn Bitmap_SetPixel(this: *mut Bitmap, x: INT, y: INT, color: *const Color) -> Status;
}
extern "C" {
    #[link_name = "\u{1}?SetResolution@Bitmap@Gdiplus@@QEAA?AW4Status@2@MM@Z"]
    pub fn Bitmap_SetResolution(this: *mut Bitmap, xdpi: REAL, ydpi: REAL) -> Status;
}
extern "C" {
    #[link_name = "\u{1}?FromDirectDrawSurface7@Bitmap@Gdiplus@@SAPEAV12@PEAUIDirectDrawSurface7@@@Z"]
    pub fn Bitmap_FromDirectDrawSurface7(surface: *mut IDirectDrawSurface7) -> *mut Bitmap;
}
extern "C" {
    #[link_name = "\u{1}?FromBITMAPINFO@Bitmap@Gdiplus@@SAPEAV12@PEBUtagBITMAPINFO@@PEAX@Z"]
    pub fn Bitmap_FromBITMAPINFO(
        gdiBitmapInfo: *const BITMAPINFO,
        gdiBitmapData: *mut c_void,
    ) -> *mut Bitmap;
}
extern "C" {
    #[link_name = "\u{1}?FromHBITMAP@Bitmap@Gdiplus@@SAPEAV12@PEAUHBITMAP__@@PEAUHPALETTE__@@@Z"]
    pub fn Bitmap_FromHBITMAP(hbm: HBITMAP, hpal: HPALETTE) -> *mut Bitmap;
}
extern "C" {
    #[link_name = "\u{1}?FromHICON@Bitmap@Gdiplus@@SAPEAV12@PEAUHICON__@@@Z"]
    pub fn Bitmap_FromHICON(hicon: HICON) -> *mut Bitmap;
}
extern "C" {
    #[link_name = "\u{1}?FromResource@Bitmap@Gdiplus@@SAPEAV12@PEAUHINSTANCE__@@PEB_W@Z"]
    pub fn Bitmap_FromResource(hInstance: HINSTANCE, bitmapName: *const WCHAR) -> *mut Bitmap;
}
extern "C" {
    #[link_name = "\u{1}?GetHBITMAP@Bitmap@Gdiplus@@QEAA?AW4Status@2@AEBVColor@2@PEAPEAUHBITMAP__@@@Z"]
    pub fn Bitmap_GetHBITMAP(
        this: *mut Bitmap,
        colorBackground: *const Color,
        hbmReturn: *mut HBITMAP,
    ) -> Status;
}
extern "C" {
    #[link_name = "\u{1}?GetHICON@Bitmap@Gdiplus@@QEAA?AW4Status@2@PEAPEAUHICON__@@@Z"]
    pub fn Bitmap_GetHICON(this: *mut Bitmap, hicon: *mut HICON) -> Status;
}
extern "C" {
    #[link_name = "\u{1}??0Bitmap@Gdiplus@@QEAA@PEB_WH@Z"]
    pub fn Bitmap_Bitmap(
        this: *mut Bitmap,
        filename: *const WCHAR,
        useEmbeddedColorManagement: BOOL,
    );
}
extern "C" {
    #[link_name = "\u{1}??0Bitmap@Gdiplus@@QEAA@PEAUIStream@@H@Z"]
    pub fn Bitmap_Bitmap1(
        this: *mut Bitmap,
        stream: *mut IStream,
        useEmbeddedColorManagement: BOOL,
    );
}
extern "C" {
    #[link_name = "\u{1}??0Bitmap@Gdiplus@@QEAA@HHHHPEAE@Z"]
    pub fn Bitmap_Bitmap2(
        this: *mut Bitmap,
        width: INT,
        height: INT,
        stride: INT,
        format: PixelFormat,
        scan0: *mut BYTE,
    );
}
extern "C" {
    #[link_name = "\u{1}??0Bitmap@Gdiplus@@QEAA@HHH@Z"]
    pub fn Bitmap_Bitmap3(this: *mut Bitmap, width: INT, height: INT, format: PixelFormat);
}
extern "C" {
    #[link_name = "\u{1}??0Bitmap@Gdiplus@@QEAA@HHPEAVGraphics@1@@Z"]
    pub fn Bitmap_Bitmap4(this: *mut Bitmap, width: INT, height: INT, target: *mut Graphics);
}
extern "C" {
    #[link_name = "\u{1}??0Bitmap@Gdiplus@@QEAA@PEAUIDirectDrawSurface7@@@Z"]
    pub fn Bitmap_Bitmap5(this: *mut Bitmap, surface: *mut IDirectDrawSurface7);
}
extern "C" {
    #[link_name = "\u{1}??0Bitmap@Gdiplus@@QEAA@PEBUtagBITMAPINFO@@PEAX@Z"]
    pub fn Bitmap_Bitmap6(
        this: *mut Bitmap,
        gdiBitmapInfo: *const BITMAPINFO,
        gdiBitmapData: *mut c_void,
    );
}
extern "C" {
    #[link_name = "\u{1}??0Bitmap@Gdiplus@@QEAA@PEAUHBITMAP__@@PEAUHPALETTE__@@@Z"]
    pub fn Bitmap_Bitmap7(this: *mut Bitmap, hbm: HBITMAP, hpal: HPALETTE);
}
extern "C" {
    #[link_name = "\u{1}??0Bitmap@Gdiplus@@QEAA@PEAUHICON__@@@Z"]
    pub fn Bitmap_Bitmap8(this: *mut Bitmap, hicon: HICON);
}
extern "C" {
    #[link_name = "\u{1}??0Bitmap@Gdiplus@@QEAA@PEAUHINSTANCE__@@PEB_W@Z"]
    pub fn Bitmap_Bitmap9(this: *mut Bitmap, hInstance: HINSTANCE, bitmapName: *const WCHAR);
}
extern "C" {
    #[link_name = "\u{1}??0Bitmap@Gdiplus@@IEAA@PEAVGpBitmap@1@@Z"]
    pub fn Bitmap_Bitmap10(this: *mut Bitmap, nativeBitmap: *mut GpBitmap);
}
impl Bitmap {
    #[inline]
    pub unsafe fn FromFile(
        filename: *const WCHAR,
        useEmbeddedColorManagement: BOOL,
    ) -> *mut Bitmap {
        Bitmap_FromFile(filename, useEmbeddedColorManagement)
    }
    #[inline]
    pub unsafe fn FromStream(
        stream: *mut IStream,
        useEmbeddedColorManagement: BOOL,
    ) -> *mut Bitmap {
        Bitmap_FromStream(stream, useEmbeddedColorManagement)
    }
    #[inline]
    pub unsafe fn Clone(&mut self, rect: *const Rect, format: PixelFormat) -> *mut Bitmap {
        Bitmap_Clone(self, rect, format)
    }
    #[inline]
    pub unsafe fn Clone1(
        &mut self,
        x: INT,
        y: INT,
        width: INT,
        height: INT,
        format: PixelFormat,
    ) -> *mut Bitmap {
        Bitmap_Clone1(self, x, y, width, height, format)
    }
    #[inline]
    pub unsafe fn Clone2(&mut self, rect: *const RectF, format: PixelFormat) -> *mut Bitmap {
        Bitmap_Clone2(self, rect, format)
    }
    #[inline]
    pub unsafe fn Clone3(
        &mut self,
        x: REAL,
        y: REAL,
        width: REAL,
        height: REAL,
        format: PixelFormat,
    ) -> *mut Bitmap {
        Bitmap_Clone3(self, x, y, width, height, format)
    }
    #[inline]
    pub unsafe fn LockBits(
        &mut self,
        rect: *const Rect,
        flags: UINT,
        format: PixelFormat,
        lockedBitmapData: *mut BitmapData,
    ) -> Status {
        Bitmap_LockBits(self, rect, flags, format, lockedBitmapData)
    }
    #[inline]
    pub unsafe fn UnlockBits(&mut self, lockedBitmapData: *mut BitmapData) -> Status {
        Bitmap_UnlockBits(self, lockedBitmapData)
    }
    #[inline]
    pub unsafe fn GetPixel(&mut self, x: INT, y: INT, color: *mut Color) -> Status {
        Bitmap_GetPixel(self, x, y, color)
    }
    #[inline]
    pub unsafe fn SetPixel(&mut self, x: INT, y: INT, color: *const Color) -> Status {
        Bitmap_SetPixel(self, x, y, color)
    }
    #[inline]
    pub unsafe fn SetResolution(&mut self, xdpi: REAL, ydpi: REAL) -> Status {
        Bitmap_SetResolution(self, xdpi, ydpi)
    }
    #[inline]
    pub unsafe fn FromDirectDrawSurface7(surface: *mut IDirectDrawSurface7) -> *mut Bitmap {
        Bitmap_FromDirectDrawSurface7(surface)
    }
    #[inline]
    pub unsafe fn FromBITMAPINFO(
        gdiBitmapInfo: *const BITMAPINFO,
        gdiBitmapData: *mut c_void,
    ) -> *mut Bitmap {
        Bitmap_FromBITMAPINFO(gdiBitmapInfo, gdiBitmapData)
    }
    #[inline]
    pub unsafe fn FromHBITMAP(hbm: HBITMAP, hpal: HPALETTE) -> *mut Bitmap {
        Bitmap_FromHBITMAP(hbm, hpal)
    }
    #[inline]
    pub unsafe fn FromHICON(hicon: HICON) -> *mut Bitmap {
        Bitmap_FromHICON(hicon)
    }
    #[inline]
    pub unsafe fn FromResource(hInstance: HINSTANCE, bitmapName: *const WCHAR) -> *mut Bitmap {
        Bitmap_FromResource(hInstance, bitmapName)
    }
    #[inline]
    pub unsafe fn GetHBITMAP(
        &mut self,
        colorBackground: *const Color,
        hbmReturn: *mut HBITMAP,
    ) -> Status {
        Bitmap_GetHBITMAP(self, colorBackground, hbmReturn)
    }
    #[inline]
    pub unsafe fn GetHICON(&mut self, hicon: *mut HICON) -> Status {
        Bitmap_GetHICON(self, hicon)
    }
    #[inline]
    pub unsafe fn new(filename: *const WCHAR, useEmbeddedColorManagement: BOOL) -> Self {
        let mut __bindgen_tmp = MaybeUninit::uninit();
        Bitmap_Bitmap(
            __bindgen_tmp.as_mut_ptr(),
            filename,
            useEmbeddedColorManagement,
        );
        __bindgen_tmp.assume_init()
    }
    #[inline]
    pub unsafe fn new1(stream: *mut IStream, useEmbeddedColorManagement: BOOL) -> Self {
        let mut __bindgen_tmp = MaybeUninit::uninit();
        Bitmap_Bitmap1(
            __bindgen_tmp.as_mut_ptr(),
            stream,
            useEmbeddedColorManagement,
        );
        __bindgen_tmp.assume_init()
    }
    #[inline]
    pub unsafe fn new2(
        width: INT,
        height: INT,
        stride: INT,
        format: PixelFormat,
        scan0: *mut BYTE,
    ) -> Self {
        let mut __bindgen_tmp = MaybeUninit::uninit();
        Bitmap_Bitmap2(
            __bindgen_tmp.as_mut_ptr(),
            width,
            height,
            stride,
            format,
            scan0,
        );
        __bindgen_tmp.assume_init()
    }
    #[inline]
    pub unsafe fn new3(width: INT, height: INT, format: PixelFormat) -> Self {
        let mut __bindgen_tmp = MaybeUninit::uninit();
        Bitmap_Bitmap3(__bindgen_tmp.as_mut_ptr(), width, height, format);
        __bindgen_tmp.assume_init()
    }
    #[inline]
    pub unsafe fn new4(width: INT, height: INT, target: *mut Graphics) -> Self {
        let mut __bindgen_tmp = MaybeUninit::uninit();
        Bitmap_Bitmap4(__bindgen_tmp.as_mut_ptr(), width, height, target);
        __bindgen_tmp.assume_init()
    }
    #[inline]
    pub unsafe fn new5(surface: *mut IDirectDrawSurface7) -> Self {
        let mut __bindgen_tmp = MaybeUninit::uninit();
        Bitmap_Bitmap5(__bindgen_tmp.as_mut_ptr(), surface);
        __bindgen_tmp.assume_init()
    }
    #[inline]
    pub unsafe fn new6(gdiBitmapInfo: *const BITMAPINFO, gdiBitmapData: *mut c_void) -> Self {
        let mut __bindgen_tmp = MaybeUninit::uninit();
        Bitmap_Bitmap6(__bindgen_tmp.as_mut_ptr(), gdiBitmapInfo, gdiBitmapData);
        __bindgen_tmp.assume_init()
    }
    #[inline]
    pub unsafe fn new7(hbm: HBITMAP, hpal: HPALETTE) -> Self {
        let mut __bindgen_tmp = MaybeUninit::uninit();
        Bitmap_Bitmap7(__bindgen_tmp.as_mut_ptr(), hbm, hpal);
        __bindgen_tmp.assume_init()
    }
    #[inline]
    pub unsafe fn new8(hicon: HICON) -> Self {
        let mut __bindgen_tmp = MaybeUninit::uninit();
        Bitmap_Bitmap8(__bindgen_tmp.as_mut_ptr(), hicon);
        __bindgen_tmp.assume_init()
    }
    #[inline]
    pub unsafe fn new9(hInstance: HINSTANCE, bitmapName: *const WCHAR) -> Self {
        let mut __bindgen_tmp = MaybeUninit::uninit();
        Bitmap_Bitmap9(__bindgen_tmp.as_mut_ptr(), hInstance, bitmapName);
        __bindgen_tmp.assume_init()
    }
    #[inline]
    pub unsafe fn new10(nativeBitmap: *mut GpBitmap) -> Self {
        let mut __bindgen_tmp = MaybeUninit::uninit();
        Bitmap_Bitmap10(__bindgen_tmp.as_mut_ptr(), nativeBitmap);
        __bindgen_tmp.assume_init()
    }
}
#[repr(C)]
pub struct CustomLineCap__bindgen_vtable(c_void);
#[repr(C)]
#[derive(Debug)]
pub struct CustomLineCap {
    pub vtable_: *const CustomLineCap__bindgen_vtable,
    pub nativeCap: *mut GpCustomLineCap,
    pub lastResult: Status,
}
extern "C" {
    #[link_name = "\u{1}?Clone@CustomLineCap@Gdiplus@@QEBAPEAV12@XZ"]
    pub fn CustomLineCap_Clone(this: *const CustomLineCap) -> *mut CustomLineCap;
}
extern "C" {
    #[link_name = "\u{1}?SetStrokeCaps@CustomLineCap@Gdiplus@@QEAA?AW4Status@2@W4LineCap@2@0@Z"]
    pub fn CustomLineCap_SetStrokeCaps(
        this: *mut CustomLineCap,
        startCap: LineCap,
        endCap: LineCap,
    ) -> Status;
}
extern "C" {
    #[link_name = "\u{1}?GetStrokeCaps@CustomLineCap@Gdiplus@@QEBA?AW4Status@2@PEAW4LineCap@2@0@Z"]
    pub fn CustomLineCap_GetStrokeCaps(
        this: *const CustomLineCap,
        startCap: *mut LineCap,
        endCap: *mut LineCap,
    ) -> Status;
}
extern "C" {
    #[link_name = "\u{1}?SetStrokeJoin@CustomLineCap@Gdiplus@@QEAA?AW4Status@2@W4LineJoin@2@@Z"]
    pub fn CustomLineCap_SetStrokeJoin(this: *mut CustomLineCap, lineJoin: LineJoin) -> Status;
}
extern "C" {
    #[link_name = "\u{1}?GetStrokeJoin@CustomLineCap@Gdiplus@@QEBA?AW4LineJoin@2@XZ"]
    pub fn CustomLineCap_GetStrokeJoin(this: *const CustomLineCap) -> LineJoin;
}
extern "C" {
    #[link_name = "\u{1}?SetBaseCap@CustomLineCap@Gdiplus@@QEAA?AW4Status@2@W4LineCap@2@@Z"]
    pub fn CustomLineCap_SetBaseCap(this: *mut CustomLineCap, baseCap: LineCap) -> Status;
}
extern "C" {
    #[link_name = "\u{1}?GetBaseCap@CustomLineCap@Gdiplus@@QEBA?AW4LineCap@2@XZ"]
    pub fn CustomLineCap_GetBaseCap(this: *const CustomLineCap) -> LineCap;
}
extern "C" {
    #[link_name = "\u{1}?SetBaseInset@CustomLineCap@Gdiplus@@QEAA?AW4Status@2@M@Z"]
    pub fn CustomLineCap_SetBaseInset(this: *mut CustomLineCap, inset: REAL) -> Status;
}
extern "C" {
    #[link_name = "\u{1}?GetBaseInset@CustomLineCap@Gdiplus@@QEBAMXZ"]
    pub fn CustomLineCap_GetBaseInset(this: *const CustomLineCap) -> REAL;
}
extern "C" {
    #[link_name = "\u{1}?SetWidthScale@CustomLineCap@Gdiplus@@QEAA?AW4Status@2@M@Z"]
    pub fn CustomLineCap_SetWidthScale(this: *mut CustomLineCap, widthScale: REAL) -> Status;
}
extern "C" {
    #[link_name = "\u{1}?GetWidthScale@CustomLineCap@Gdiplus@@QEBAMXZ"]
    pub fn CustomLineCap_GetWidthScale(this: *const CustomLineCap) -> REAL;
}
extern "C" {
    #[link_name = "\u{1}?GetLastStatus@CustomLineCap@Gdiplus@@QEBA?AW4Status@2@XZ"]
    pub fn CustomLineCap_GetLastStatus(this: *const CustomLineCap) -> Status;
}
extern "C" {
    #[link_name = "\u{1}??0CustomLineCap@Gdiplus@@QEAA@PEBVGraphicsPath@1@0W4LineCap@1@M@Z"]
    pub fn CustomLineCap_CustomLineCap(
        this: *mut CustomLineCap,
        fillPath: *const GraphicsPath,
        strokePath: *const GraphicsPath,
        baseCap: LineCap,
        baseInset: REAL,
    );
}
extern "C" {
    #[link_name = "\u{1}??0CustomLineCap@Gdiplus@@IEAA@XZ"]
    pub fn CustomLineCap_CustomLineCap1(this: *mut CustomLineCap);
}
impl CustomLineCap {
    #[inline]
    pub unsafe fn Clone(&self) -> *mut CustomLineCap {
        CustomLineCap_Clone(self)
    }
    #[inline]
    pub unsafe fn SetStrokeCaps(&mut self, startCap: LineCap, endCap: LineCap) -> Status {
        CustomLineCap_SetStrokeCaps(self, startCap, endCap)
    }
    #[inline]
    pub unsafe fn GetStrokeCaps(&self, startCap: *mut LineCap, endCap: *mut LineCap) -> Status {
        CustomLineCap_GetStrokeCaps(self, startCap, endCap)
    }
    #[inline]
    pub unsafe fn SetStrokeJoin(&mut self, lineJoin: LineJoin) -> Status {
        CustomLineCap_SetStrokeJoin(self, lineJoin)
    }
    #[inline]
    pub unsafe fn GetStrokeJoin(&self) -> LineJoin {
        CustomLineCap_GetStrokeJoin(self)
    }
    #[inline]
    pub unsafe fn SetBaseCap(&mut self, baseCap: LineCap) -> Status {
        CustomLineCap_SetBaseCap(self, baseCap)
    }
    #[inline]
    pub unsafe fn GetBaseCap(&self) -> LineCap {
        CustomLineCap_GetBaseCap(self)
    }
    #[inline]
    pub unsafe fn SetBaseInset(&mut self, inset: REAL) -> Status {
        CustomLineCap_SetBaseInset(self, inset)
    }
    #[inline]
    pub unsafe fn GetBaseInset(&self) -> REAL {
        CustomLineCap_GetBaseInset(self)
    }
    #[inline]
    pub unsafe fn SetWidthScale(&mut self, widthScale: REAL) -> Status {
        CustomLineCap_SetWidthScale(self, widthScale)
    }
    #[inline]
    pub unsafe fn GetWidthScale(&self) -> REAL {
        CustomLineCap_GetWidthScale(self)
    }
    #[inline]
    pub unsafe fn GetLastStatus(&self) -> Status {
        CustomLineCap_GetLastStatus(self)
    }
    #[inline]
    pub unsafe fn new(
        fillPath: *const GraphicsPath,
        strokePath: *const GraphicsPath,
        baseCap: LineCap,
        baseInset: REAL,
    ) -> Self {
        let mut __bindgen_tmp = MaybeUninit::uninit();
        CustomLineCap_CustomLineCap(
            __bindgen_tmp.as_mut_ptr(),
            fillPath,
            strokePath,
            baseCap,
            baseInset,
        );
        __bindgen_tmp.assume_init()
    }
    #[inline]
    pub unsafe fn new1() -> Self {
        let mut __bindgen_tmp = MaybeUninit::uninit();
        CustomLineCap_CustomLineCap1(__bindgen_tmp.as_mut_ptr());
        __bindgen_tmp.assume_init()
    }
}
extern "C" {
    #[link_name = "\u{1}??_DCustomLineCap@Gdiplus@@QEAAXXZ"]
    pub fn CustomLineCap_CustomLineCap_destructor(this: *mut CustomLineCap);
}
#[repr(C)]
pub struct CachedBitmap__bindgen_vtable(c_void);
#[repr(C)]
#[derive(Debug)]
pub struct CachedBitmap {
    pub vtable_: *const CachedBitmap__bindgen_vtable,
    pub nativeCachedBitmap: *mut GpCachedBitmap,
    pub lastResult: Status,
}
extern "C" {
    #[link_name = "\u{1}?GetLastStatus@CachedBitmap@Gdiplus@@QEBA?AW4Status@2@XZ"]
    pub fn CachedBitmap_GetLastStatus(this: *const CachedBitmap) -> Status;
}
extern "C" {
    #[link_name = "\u{1}??0CachedBitmap@Gdiplus@@QEAA@PEAVBitmap@1@PEAVGraphics@1@@Z"]
    pub fn CachedBitmap_CachedBitmap(
        this: *mut CachedBitmap,
        bitmap: *mut Bitmap,
        graphics: *mut Graphics,
    );
}
impl CachedBitmap {
    #[inline]
    pub unsafe fn GetLastStatus(&self) -> Status {
        CachedBitmap_GetLastStatus(self)
    }
    #[inline]
    pub unsafe fn new(bitmap: *mut Bitmap, graphics: *mut Graphics) -> Self {
        let mut __bindgen_tmp = MaybeUninit::uninit();
        CachedBitmap_CachedBitmap(__bindgen_tmp.as_mut_ptr(), bitmap, graphics);
        __bindgen_tmp.assume_init()
    }
}
extern "C" {
    #[link_name = "\u{1}??_DCachedBitmap@Gdiplus@@QEAAXXZ"]
    pub fn CachedBitmap_CachedBitmap_destructor(this: *mut CachedBitmap);
}
#[repr(C)]
#[derive(Debug)]
pub struct Metafile {
    pub _base: Image,
}
extern "C" {
    #[link_name = "\u{1}?GetMetafileHeader@Metafile@Gdiplus@@SA?AW4Status@2@PEAUHMETAFILE__@@PEBUWmfPlaceableFileHeader@2@PEAVMetafileHeader@2@@Z"]
    pub fn Metafile_GetMetafileHeader(
        hWmf: HMETAFILE,
        wmfPlaceableFileHeader: *const WmfPlaceableFileHeader,
        header: *mut MetafileHeader,
    ) -> Status;
}
extern "C" {
    #[link_name = "\u{1}?GetMetafileHeader@Metafile@Gdiplus@@SA?AW4Status@2@PEAUHENHMETAFILE__@@PEAVMetafileHeader@2@@Z"]
    pub fn Metafile_GetMetafileHeader1(hEmf: HENHMETAFILE, header: *mut MetafileHeader) -> Status;
}
extern "C" {
    #[link_name = "\u{1}?GetMetafileHeader@Metafile@Gdiplus@@SA?AW4Status@2@PEB_WPEAVMetafileHeader@2@@Z"]
    pub fn Metafile_GetMetafileHeader2(
        filename: *const WCHAR,
        header: *mut MetafileHeader,
    ) -> Status;
}
extern "C" {
    #[link_name = "\u{1}?GetMetafileHeader@Metafile@Gdiplus@@SA?AW4Status@2@PEAUIStream@@PEAVMetafileHeader@2@@Z"]
    pub fn Metafile_GetMetafileHeader3(stream: *mut IStream, header: *mut MetafileHeader)
        -> Status;
}
extern "C" {
    #[link_name = "\u{1}?GetMetafileHeader@Metafile@Gdiplus@@QEBA?AW4Status@2@PEAVMetafileHeader@2@@Z"]
    pub fn Metafile_GetMetafileHeader4(
        this: *const Metafile,
        header: *mut MetafileHeader,
    ) -> Status;
}
extern "C" {
    #[link_name = "\u{1}?GetHENHMETAFILE@Metafile@Gdiplus@@QEAAPEAUHENHMETAFILE__@@XZ"]
    pub fn Metafile_GetHENHMETAFILE(this: *mut Metafile) -> HENHMETAFILE;
}
extern "C" {
    #[link_name = "\u{1}?PlayRecord@Metafile@Gdiplus@@QEBA?AW4Status@2@W4EmfPlusRecordType@2@IIPEBE@Z"]
    pub fn Metafile_PlayRecord(
        this: *const Metafile,
        recordType: EmfPlusRecordType,
        flags: UINT,
        dataSize: UINT,
        data: *const BYTE,
    ) -> Status;
}
extern "C" {
    #[link_name = "\u{1}?SetDownLevelRasterizationLimit@Metafile@Gdiplus@@QEAA?AW4Status@2@I@Z"]
    pub fn Metafile_SetDownLevelRasterizationLimit(
        this: *mut Metafile,
        metafileRasterizationLimitDpi: UINT,
    ) -> Status;
}
extern "C" {
    #[link_name = "\u{1}?GetDownLevelRasterizationLimit@Metafile@Gdiplus@@QEBAIXZ"]
    pub fn Metafile_GetDownLevelRasterizationLimit(this: *const Metafile) -> UINT;
}
extern "C" {
    #[link_name = "\u{1}?EmfToWmfBits@Metafile@Gdiplus@@SAIPEAUHENHMETAFILE__@@IPEAEHH@Z"]
    pub fn Metafile_EmfToWmfBits(
        hemf: HENHMETAFILE,
        cbData16: UINT,
        pData16: LPBYTE,
        iMapMode: INT,
        eFlags: INT,
    ) -> UINT;
}
extern "C" {
    #[link_name = "\u{1}??0Metafile@Gdiplus@@QEAA@PEAUHMETAFILE__@@PEBUWmfPlaceableFileHeader@1@H@Z"]
    pub fn Metafile_Metafile(
        this: *mut Metafile,
        hWmf: HMETAFILE,
        wmfPlaceableFileHeader: *const WmfPlaceableFileHeader,
        deleteWmf: BOOL,
    );
}
extern "C" {
    #[link_name = "\u{1}??0Metafile@Gdiplus@@QEAA@PEAUHENHMETAFILE__@@H@Z"]
    pub fn Metafile_Metafile1(this: *mut Metafile, hEmf: HENHMETAFILE, deleteEmf: BOOL);
}
extern "C" {
    #[link_name = "\u{1}??0Metafile@Gdiplus@@QEAA@PEB_W@Z"]
    pub fn Metafile_Metafile2(this: *mut Metafile, filename: *const WCHAR);
}
extern "C" {
    #[link_name = "\u{1}??0Metafile@Gdiplus@@QEAA@PEB_WPEBUWmfPlaceableFileHeader@1@@Z"]
    pub fn Metafile_Metafile3(
        this: *mut Metafile,
        filename: *const WCHAR,
        wmfPlaceableFileHeader: *const WmfPlaceableFileHeader,
    );
}
extern "C" {
    #[link_name = "\u{1}??0Metafile@Gdiplus@@QEAA@PEAUIStream@@@Z"]
    pub fn Metafile_Metafile4(this: *mut Metafile, stream: *mut IStream);
}
extern "C" {
    #[link_name = "\u{1}??0Metafile@Gdiplus@@QEAA@PEAUHDC__@@W4EmfType@1@PEB_W@Z"]
    pub fn Metafile_Metafile5(
        this: *mut Metafile,
        referenceHdc: HDC,
        type_: EmfType,
        description: *const WCHAR,
    );
}
extern "C" {
    #[link_name = "\u{1}??0Metafile@Gdiplus@@QEAA@PEAUHDC__@@AEBVRectF@1@W4MetafileFrameUnit@1@W4EmfType@1@PEB_W@Z"]
    pub fn Metafile_Metafile6(
        this: *mut Metafile,
        referenceHdc: HDC,
        frameRect: *const RectF,
        frameUnit: MetafileFrameUnit,
        type_: EmfType,
        description: *const WCHAR,
    );
}
extern "C" {
    #[link_name = "\u{1}??0Metafile@Gdiplus@@QEAA@PEAUHDC__@@AEBVRect@1@W4MetafileFrameUnit@1@W4EmfType@1@PEB_W@Z"]
    pub fn Metafile_Metafile7(
        this: *mut Metafile,
        referenceHdc: HDC,
        frameRect: *const Rect,
        frameUnit: MetafileFrameUnit,
        type_: EmfType,
        description: *const WCHAR,
    );
}
extern "C" {
    #[link_name = "\u{1}??0Metafile@Gdiplus@@QEAA@PEB_WPEAUHDC__@@W4EmfType@1@0@Z"]
    pub fn Metafile_Metafile8(
        this: *mut Metafile,
        fileName: *const WCHAR,
        referenceHdc: HDC,
        type_: EmfType,
        description: *const WCHAR,
    );
}
extern "C" {
    #[link_name = "\u{1}??0Metafile@Gdiplus@@QEAA@PEB_WPEAUHDC__@@AEBVRectF@1@W4MetafileFrameUnit@1@W4EmfType@1@0@Z"]
    pub fn Metafile_Metafile9(
        this: *mut Metafile,
        fileName: *const WCHAR,
        referenceHdc: HDC,
        frameRect: *const RectF,
        frameUnit: MetafileFrameUnit,
        type_: EmfType,
        description: *const WCHAR,
    );
}
extern "C" {
    #[link_name = "\u{1}??0Metafile@Gdiplus@@QEAA@PEB_WPEAUHDC__@@AEBVRect@1@W4MetafileFrameUnit@1@W4EmfType@1@0@Z"]
    pub fn Metafile_Metafile10(
        this: *mut Metafile,
        fileName: *const WCHAR,
        referenceHdc: HDC,
        frameRect: *const Rect,
        frameUnit: MetafileFrameUnit,
        type_: EmfType,
        description: *const WCHAR,
    );
}
extern "C" {
    #[link_name = "\u{1}??0Metafile@Gdiplus@@QEAA@PEAUIStream@@PEAUHDC__@@W4EmfType@1@PEB_W@Z"]
    pub fn Metafile_Metafile11(
        this: *mut Metafile,
        stream: *mut IStream,
        referenceHdc: HDC,
        type_: EmfType,
        description: *const WCHAR,
    );
}
extern "C" {
    #[link_name = "\u{1}??0Metafile@Gdiplus@@QEAA@PEAUIStream@@PEAUHDC__@@AEBVRectF@1@W4MetafileFrameUnit@1@W4EmfType@1@PEB_W@Z"]
    pub fn Metafile_Metafile12(
        this: *mut Metafile,
        stream: *mut IStream,
        referenceHdc: HDC,
        frameRect: *const RectF,
        frameUnit: MetafileFrameUnit,
        type_: EmfType,
        description: *const WCHAR,
    );
}
extern "C" {
    #[link_name = "\u{1}??0Metafile@Gdiplus@@QEAA@PEAUIStream@@PEAUHDC__@@AEBVRect@1@W4MetafileFrameUnit@1@W4EmfType@1@PEB_W@Z"]
    pub fn Metafile_Metafile13(
        this: *mut Metafile,
        stream: *mut IStream,
        referenceHdc: HDC,
        frameRect: *const Rect,
        frameUnit: MetafileFrameUnit,
        type_: EmfType,
        description: *const WCHAR,
    );
}
impl Metafile {
    #[inline]
    pub unsafe fn GetMetafileHeader(
        hWmf: HMETAFILE,
        wmfPlaceableFileHeader: *const WmfPlaceableFileHeader,
        header: *mut MetafileHeader,
    ) -> Status {
        Metafile_GetMetafileHeader(hWmf, wmfPlaceableFileHeader, header)
    }
    #[inline]
    pub unsafe fn GetMetafileHeader1(hEmf: HENHMETAFILE, header: *mut MetafileHeader) -> Status {
        Metafile_GetMetafileHeader1(hEmf, header)
    }
    #[inline]
    pub unsafe fn GetMetafileHeader2(
        filename: *const WCHAR,
        header: *mut MetafileHeader,
    ) -> Status {
        Metafile_GetMetafileHeader2(filename, header)
    }
    #[inline]
    pub unsafe fn GetMetafileHeader3(stream: *mut IStream, header: *mut MetafileHeader) -> Status {
        Metafile_GetMetafileHeader3(stream, header)
    }
    #[inline]
    pub unsafe fn GetMetafileHeader4(&self, header: *mut MetafileHeader) -> Status {
        Metafile_GetMetafileHeader4(self, header)
    }
    #[inline]
    pub unsafe fn GetHENHMETAFILE(&mut self) -> HENHMETAFILE {
        Metafile_GetHENHMETAFILE(self)
    }
    #[inline]
    pub unsafe fn PlayRecord(
        &self,
        recordType: EmfPlusRecordType,
        flags: UINT,
        dataSize: UINT,
        data: *const BYTE,
    ) -> Status {
        Metafile_PlayRecord(self, recordType, flags, dataSize, data)
    }
    #[inline]
    pub unsafe fn SetDownLevelRasterizationLimit(
        &mut self,
        metafileRasterizationLimitDpi: UINT,
    ) -> Status {
        Metafile_SetDownLevelRasterizationLimit(self, metafileRasterizationLimitDpi)
    }
    #[inline]
    pub unsafe fn GetDownLevelRasterizationLimit(&self) -> UINT {
        Metafile_GetDownLevelRasterizationLimit(self)
    }
    #[inline]
    pub unsafe fn EmfToWmfBits(
        hemf: HENHMETAFILE,
        cbData16: UINT,
        pData16: LPBYTE,
        iMapMode: INT,
        eFlags: INT,
    ) -> UINT {
        Metafile_EmfToWmfBits(hemf, cbData16, pData16, iMapMode, eFlags)
    }
    #[inline]
    pub unsafe fn new(
        hWmf: HMETAFILE,
        wmfPlaceableFileHeader: *const WmfPlaceableFileHeader,
        deleteWmf: BOOL,
    ) -> Self {
        let mut __bindgen_tmp = MaybeUninit::uninit();
        Metafile_Metafile(
            __bindgen_tmp.as_mut_ptr(),
            hWmf,
            wmfPlaceableFileHeader,
            deleteWmf,
        );
        __bindgen_tmp.assume_init()
    }
    #[inline]
    pub unsafe fn new1(hEmf: HENHMETAFILE, deleteEmf: BOOL) -> Self {
        let mut __bindgen_tmp = MaybeUninit::uninit();
        Metafile_Metafile1(__bindgen_tmp.as_mut_ptr(), hEmf, deleteEmf);
        __bindgen_tmp.assume_init()
    }
    #[inline]
    pub unsafe fn new2(filename: *const WCHAR) -> Self {
        let mut __bindgen_tmp = MaybeUninit::uninit();
        Metafile_Metafile2(__bindgen_tmp.as_mut_ptr(), filename);
        __bindgen_tmp.assume_init()
    }
    #[inline]
    pub unsafe fn new3(
        filename: *const WCHAR,
        wmfPlaceableFileHeader: *const WmfPlaceableFileHeader,
    ) -> Self {
        let mut __bindgen_tmp = MaybeUninit::uninit();
        Metafile_Metafile3(__bindgen_tmp.as_mut_ptr(), filename, wmfPlaceableFileHeader);
        __bindgen_tmp.assume_init()
    }
    #[inline]
    pub unsafe fn new4(stream: *mut IStream) -> Self {
        let mut __bindgen_tmp = MaybeUninit::uninit();
        Metafile_Metafile4(__bindgen_tmp.as_mut_ptr(), stream);
        __bindgen_tmp.assume_init()
    }
    #[inline]
    pub unsafe fn new5(referenceHdc: HDC, type_: EmfType, description: *const WCHAR) -> Self {
        let mut __bindgen_tmp = MaybeUninit::uninit();
        Metafile_Metafile5(__bindgen_tmp.as_mut_ptr(), referenceHdc, type_, description);
        __bindgen_tmp.assume_init()
    }
    #[inline]
    pub unsafe fn new6(
        referenceHdc: HDC,
        frameRect: *const RectF,
        frameUnit: MetafileFrameUnit,
        type_: EmfType,
        description: *const WCHAR,
    ) -> Self {
        let mut __bindgen_tmp = MaybeUninit::uninit();
        Metafile_Metafile6(
            __bindgen_tmp.as_mut_ptr(),
            referenceHdc,
            frameRect,
            frameUnit,
            type_,
            description,
        );
        __bindgen_tmp.assume_init()
    }
    #[inline]
    pub unsafe fn new7(
        referenceHdc: HDC,
        frameRect: *const Rect,
        frameUnit: MetafileFrameUnit,
        type_: EmfType,
        description: *const WCHAR,
    ) -> Self {
        let mut __bindgen_tmp = MaybeUninit::uninit();
        Metafile_Metafile7(
            __bindgen_tmp.as_mut_ptr(),
            referenceHdc,
            frameRect,
            frameUnit,
            type_,
            description,
        );
        __bindgen_tmp.assume_init()
    }
    #[inline]
    pub unsafe fn new8(
        fileName: *const WCHAR,
        referenceHdc: HDC,
        type_: EmfType,
        description: *const WCHAR,
    ) -> Self {
        let mut __bindgen_tmp = MaybeUninit::uninit();
        Metafile_Metafile8(
            __bindgen_tmp.as_mut_ptr(),
            fileName,
            referenceHdc,
            type_,
            description,
        );
        __bindgen_tmp.assume_init()
    }
    #[inline]
    pub unsafe fn new9(
        fileName: *const WCHAR,
        referenceHdc: HDC,
        frameRect: *const RectF,
        frameUnit: MetafileFrameUnit,
        type_: EmfType,
        description: *const WCHAR,
    ) -> Self {
        let mut __bindgen_tmp = MaybeUninit::uninit();
        Metafile_Metafile9(
            __bindgen_tmp.as_mut_ptr(),
            fileName,
            referenceHdc,
            frameRect,
            frameUnit,
            type_,
            description,
        );
        __bindgen_tmp.assume_init()
    }
    #[inline]
    pub unsafe fn new10(
        fileName: *const WCHAR,
        referenceHdc: HDC,
        frameRect: *const Rect,
        frameUnit: MetafileFrameUnit,
        type_: EmfType,
        description: *const WCHAR,
    ) -> Self {
        let mut __bindgen_tmp = MaybeUninit::uninit();
        Metafile_Metafile10(
            __bindgen_tmp.as_mut_ptr(),
            fileName,
            referenceHdc,
            frameRect,
            frameUnit,
            type_,
            description,
        );
        __bindgen_tmp.assume_init()
    }
    #[inline]
    pub unsafe fn new11(
        stream: *mut IStream,
        referenceHdc: HDC,
        type_: EmfType,
        description: *const WCHAR,
    ) -> Self {
        let mut __bindgen_tmp = MaybeUninit::uninit();
        Metafile_Metafile11(
            __bindgen_tmp.as_mut_ptr(),
            stream,
            referenceHdc,
            type_,
            description,
        );
        __bindgen_tmp.assume_init()
    }
    #[inline]
    pub unsafe fn new12(
        stream: *mut IStream,
        referenceHdc: HDC,
        frameRect: *const RectF,
        frameUnit: MetafileFrameUnit,
        type_: EmfType,
        description: *const WCHAR,
    ) -> Self {
        let mut __bindgen_tmp = MaybeUninit::uninit();
        Metafile_Metafile12(
            __bindgen_tmp.as_mut_ptr(),
            stream,
            referenceHdc,
            frameRect,
            frameUnit,
            type_,
            description,
        );
        __bindgen_tmp.assume_init()
    }
    #[inline]
    pub unsafe fn new13(
        stream: *mut IStream,
        referenceHdc: HDC,
        frameRect: *const Rect,
        frameUnit: MetafileFrameUnit,
        type_: EmfType,
        description: *const WCHAR,
    ) -> Self {
        let mut __bindgen_tmp = MaybeUninit::uninit();
        Metafile_Metafile13(
            __bindgen_tmp.as_mut_ptr(),
            stream,
            referenceHdc,
            frameRect,
            frameUnit,
            type_,
            description,
        );
        __bindgen_tmp.assume_init()
    }
}
#[repr(C)]
#[derive(Debug)]
pub struct Matrix {
    pub nativeMatrix: *mut GpMatrix,
    pub lastResult: Status,
}
#[repr(C)]
#[derive(Debug)]
pub struct Pen {
    pub nativePen: *mut GpPen,
    pub lastResult: Status,
}
#[repr(C)]
#[derive(Debug)]
pub struct StringFormat {
    pub nativeFormat: *mut GpStringFormat,
    pub lastError: Status,
}
extern "C" {
    #[link_name = "\u{1}?GenericDefault@StringFormat@Gdiplus@@SAPEBV12@XZ"]
    pub fn StringFormat_GenericDefault() -> *const StringFormat;
}
extern "C" {
    #[link_name = "\u{1}?GenericTypographic@StringFormat@Gdiplus@@SAPEBV12@XZ"]
    pub fn StringFormat_GenericTypographic() -> *const StringFormat;
}
impl StringFormat {
    #[inline]
    pub unsafe fn GenericDefault() -> *const StringFormat {
        StringFormat_GenericDefault()
    }
    #[inline]
    pub unsafe fn GenericTypographic() -> *const StringFormat {
        StringFormat_GenericTypographic()
    }
}
#[repr(C)]
#[derive(Debug)]
pub struct GraphicsPath {
    pub nativePath: *mut GpPath,
    pub lastResult: Status,
}
extern "C" {
    #[link_name = "\u{1}?GetBounds@GraphicsPath@Gdiplus@@QEBA?AW4Status@2@PEAVRectF@2@PEBVMatrix@2@PEBVPen@2@@Z"]
    pub fn GraphicsPath_GetBounds(
        this: *const GraphicsPath,
        bounds: *mut RectF,
        matrix: *const Matrix,
        pen: *const Pen,
    ) -> Status;
}
extern "C" {
    #[link_name = "\u{1}?GetBounds@GraphicsPath@Gdiplus@@QEBA?AW4Status@2@PEAVRect@2@PEBVMatrix@2@PEBVPen@2@@Z"]
    pub fn GraphicsPath_GetBounds1(
        this: *const GraphicsPath,
        bounds: *mut Rect,
        matrix: *const Matrix,
        pen: *const Pen,
    ) -> Status;
}
extern "C" {
    #[link_name = "\u{1}?IsVisible@GraphicsPath@Gdiplus@@QEBAHMMPEBVGraphics@2@@Z"]
    pub fn GraphicsPath_IsVisible(
        this: *const GraphicsPath,
        x: REAL,
        y: REAL,
        g: *const Graphics,
    ) -> BOOL;
}
extern "C" {
    #[link_name = "\u{1}?IsVisible@GraphicsPath@Gdiplus@@QEBAHHHPEBVGraphics@2@@Z"]
    pub fn GraphicsPath_IsVisible1(
        this: *const GraphicsPath,
        x: INT,
        y: INT,
        g: *const Graphics,
    ) -> BOOL;
}
extern "C" {
    #[link_name = "\u{1}?IsOutlineVisible@GraphicsPath@Gdiplus@@QEBAHMMPEBVPen@2@PEBVGraphics@2@@Z"]
    pub fn GraphicsPath_IsOutlineVisible(
        this: *const GraphicsPath,
        x: REAL,
        y: REAL,
        pen: *const Pen,
        g: *const Graphics,
    ) -> BOOL;
}
extern "C" {
    #[link_name = "\u{1}?IsOutlineVisible@GraphicsPath@Gdiplus@@QEBAHHHPEBVPen@2@PEBVGraphics@2@@Z"]
    pub fn GraphicsPath_IsOutlineVisible1(
        this: *const GraphicsPath,
        x: INT,
        y: INT,
        pen: *const Pen,
        g: *const Graphics,
    ) -> BOOL;
}
impl GraphicsPath {
    #[inline]
    pub unsafe fn GetBounds(
        &self,
        bounds: *mut RectF,
        matrix: *const Matrix,
        pen: *const Pen,
    ) -> Status {
        GraphicsPath_GetBounds(self, bounds, matrix, pen)
    }
    #[inline]
    pub unsafe fn GetBounds1(
        &self,
        bounds: *mut Rect,
        matrix: *const Matrix,
        pen: *const Pen,
    ) -> Status {
        GraphicsPath_GetBounds1(self, bounds, matrix, pen)
    }
    #[inline]
    pub unsafe fn IsVisible(&self, x: REAL, y: REAL, g: *const Graphics) -> BOOL {
        GraphicsPath_IsVisible(self, x, y, g)
    }
    #[inline]
    pub unsafe fn IsVisible1(&self, x: INT, y: INT, g: *const Graphics) -> BOOL {
        GraphicsPath_IsVisible1(self, x, y, g)
    }
    #[inline]
    pub unsafe fn IsOutlineVisible(
        &self,
        x: REAL,
        y: REAL,
        pen: *const Pen,
        g: *const Graphics,
    ) -> BOOL {
        GraphicsPath_IsOutlineVisible(self, x, y, pen, g)
    }
    #[inline]
    pub unsafe fn IsOutlineVisible1(
        &self,
        x: INT,
        y: INT,
        pen: *const Pen,
        g: *const Graphics,
    ) -> BOOL {
        GraphicsPath_IsOutlineVisible1(self, x, y, pen, g)
    }
}
#[repr(C)]
#[derive(Debug)]
pub struct Graphics {
    pub nativeGraphics: *mut GpGraphics,
    pub lastResult: Status,
}
