#![allow(deref_nullptr, unaligned_references)]

use crate::*;

#[test]
fn bindgen_test_layout_GdiplusBase() {
    assert_eq!(
        ::std::mem::size_of::<GdiplusBase>(),
        1usize,
        concat!("Size of: ", stringify!(GdiplusBase))
    );
    assert_eq!(
        ::std::mem::align_of::<GdiplusBase>(),
        1usize,
        concat!("Alignment of ", stringify!(GdiplusBase))
    );
}
#[test]
fn bindgen_test_layout_SizeF() {
    assert_eq!(
        ::std::mem::size_of::<SizeF>(),
        8usize,
        concat!("Size of: ", stringify!(SizeF))
    );
    assert_eq!(
        ::std::mem::align_of::<SizeF>(),
        4usize,
        concat!("Alignment of ", stringify!(SizeF))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<SizeF>())).Width as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(SizeF),
            "::",
            stringify!(Width)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<SizeF>())).Height as *const _ as usize },
        4usize,
        concat!(
            "Offset of field: ",
            stringify!(SizeF),
            "::",
            stringify!(Height)
        )
    );
}
#[test]
fn bindgen_test_layout_PointF() {
    assert_eq!(
        ::std::mem::size_of::<PointF>(),
        8usize,
        concat!("Size of: ", stringify!(PointF))
    );
    assert_eq!(
        ::std::mem::align_of::<PointF>(),
        4usize,
        concat!("Alignment of ", stringify!(PointF))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<PointF>())).X as *const _ as usize },
        0usize,
        concat!("Offset of field: ", stringify!(PointF), "::", stringify!(X))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<PointF>())).Y as *const _ as usize },
        4usize,
        concat!("Offset of field: ", stringify!(PointF), "::", stringify!(Y))
    );
}
#[test]
fn bindgen_test_layout_Point() {
    assert_eq!(
        ::std::mem::size_of::<Point>(),
        8usize,
        concat!("Size of: ", stringify!(Point))
    );
    assert_eq!(
        ::std::mem::align_of::<Point>(),
        4usize,
        concat!("Alignment of ", stringify!(Point))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<Point>())).X as *const _ as usize },
        0usize,
        concat!("Offset of field: ", stringify!(Point), "::", stringify!(X))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<Point>())).Y as *const _ as usize },
        4usize,
        concat!("Offset of field: ", stringify!(Point), "::", stringify!(Y))
    );
}
#[test]
fn bindgen_test_layout_RectF() {
    assert_eq!(
        ::std::mem::size_of::<RectF>(),
        16usize,
        concat!("Size of: ", stringify!(RectF))
    );
    assert_eq!(
        ::std::mem::align_of::<RectF>(),
        4usize,
        concat!("Alignment of ", stringify!(RectF))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<RectF>())).X as *const _ as usize },
        0usize,
        concat!("Offset of field: ", stringify!(RectF), "::", stringify!(X))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<RectF>())).Y as *const _ as usize },
        4usize,
        concat!("Offset of field: ", stringify!(RectF), "::", stringify!(Y))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<RectF>())).Width as *const _ as usize },
        8usize,
        concat!(
            "Offset of field: ",
            stringify!(RectF),
            "::",
            stringify!(Width)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<RectF>())).Height as *const _ as usize },
        12usize,
        concat!(
            "Offset of field: ",
            stringify!(RectF),
            "::",
            stringify!(Height)
        )
    );
}
#[test]
fn bindgen_test_layout_Rect() {
    assert_eq!(
        ::std::mem::size_of::<Rect>(),
        16usize,
        concat!("Size of: ", stringify!(Rect))
    );
    assert_eq!(
        ::std::mem::align_of::<Rect>(),
        4usize,
        concat!("Alignment of ", stringify!(Rect))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<Rect>())).X as *const _ as usize },
        0usize,
        concat!("Offset of field: ", stringify!(Rect), "::", stringify!(X))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<Rect>())).Y as *const _ as usize },
        4usize,
        concat!("Offset of field: ", stringify!(Rect), "::", stringify!(Y))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<Rect>())).Width as *const _ as usize },
        8usize,
        concat!(
            "Offset of field: ",
            stringify!(Rect),
            "::",
            stringify!(Width)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<Rect>())).Height as *const _ as usize },
        12usize,
        concat!(
            "Offset of field: ",
            stringify!(Rect),
            "::",
            stringify!(Height)
        )
    );
}
#[test]
fn bindgen_test_layout_PathData() {
    assert_eq!(
        ::std::mem::size_of::<PathData>(),
        24usize,
        concat!("Size of: ", stringify!(PathData))
    );
    assert_eq!(
        ::std::mem::align_of::<PathData>(),
        8usize,
        concat!("Alignment of ", stringify!(PathData))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<PathData>())).Count as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(PathData),
            "::",
            stringify!(Count)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<PathData>())).Points as *const _ as usize },
        8usize,
        concat!(
            "Offset of field: ",
            stringify!(PathData),
            "::",
            stringify!(Points)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<PathData>())).Types as *const _ as usize },
        16usize,
        concat!(
            "Offset of field: ",
            stringify!(PathData),
            "::",
            stringify!(Types)
        )
    );
}
#[test]
fn bindgen_test_layout_CharacterRange() {
    assert_eq!(
        ::std::mem::size_of::<CharacterRange>(),
        8usize,
        concat!("Size of: ", stringify!(CharacterRange))
    );
    assert_eq!(
        ::std::mem::align_of::<CharacterRange>(),
        4usize,
        concat!("Alignment of ", stringify!(CharacterRange))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<CharacterRange>())).First as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(CharacterRange),
            "::",
            stringify!(First)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<CharacterRange>())).Length as *const _ as usize },
        4usize,
        concat!(
            "Offset of field: ",
            stringify!(CharacterRange),
            "::",
            stringify!(Length)
        )
    );
}
#[test]
fn bindgen_test_layout_GdiplusStartupInput() {
    assert_eq!(
        ::std::mem::size_of::<GdiplusStartupInput>(),
        24usize,
        concat!("Size of: ", stringify!(GdiplusStartupInput))
    );
    assert_eq!(
        ::std::mem::align_of::<GdiplusStartupInput>(),
        8usize,
        concat!("Alignment of ", stringify!(GdiplusStartupInput))
    );
    assert_eq!(
        unsafe {
            &(*(::std::ptr::null::<GdiplusStartupInput>())).GdiplusVersion as *const _ as usize
        },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(GdiplusStartupInput),
            "::",
            stringify!(GdiplusVersion)
        )
    );
    assert_eq!(
        unsafe {
            &(*(::std::ptr::null::<GdiplusStartupInput>())).DebugEventCallback as *const _ as usize
        },
        8usize,
        concat!(
            "Offset of field: ",
            stringify!(GdiplusStartupInput),
            "::",
            stringify!(DebugEventCallback)
        )
    );
    assert_eq!(
        unsafe {
            &(*(::std::ptr::null::<GdiplusStartupInput>())).SuppressBackgroundThread as *const _
                as usize
        },
        16usize,
        concat!(
            "Offset of field: ",
            stringify!(GdiplusStartupInput),
            "::",
            stringify!(SuppressBackgroundThread)
        )
    );
    assert_eq!(
        unsafe {
            &(*(::std::ptr::null::<GdiplusStartupInput>())).SuppressExternalCodecs as *const _
                as usize
        },
        20usize,
        concat!(
            "Offset of field: ",
            stringify!(GdiplusStartupInput),
            "::",
            stringify!(SuppressExternalCodecs)
        )
    );
}
#[test]
fn bindgen_test_layout_GdiplusStartupOutput() {
    assert_eq!(
        ::std::mem::size_of::<GdiplusStartupOutput>(),
        16usize,
        concat!("Size of: ", stringify!(GdiplusStartupOutput))
    );
    assert_eq!(
        ::std::mem::align_of::<GdiplusStartupOutput>(),
        8usize,
        concat!("Alignment of ", stringify!(GdiplusStartupOutput))
    );
    assert_eq!(
        unsafe {
            &(*(::std::ptr::null::<GdiplusStartupOutput>())).NotificationHook as *const _ as usize
        },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(GdiplusStartupOutput),
            "::",
            stringify!(NotificationHook)
        )
    );
    assert_eq!(
        unsafe {
            &(*(::std::ptr::null::<GdiplusStartupOutput>())).NotificationUnhook as *const _ as usize
        },
        8usize,
        concat!(
            "Offset of field: ",
            stringify!(GdiplusStartupOutput),
            "::",
            stringify!(NotificationUnhook)
        )
    );
}
#[test]
fn bindgen_test_layout_ColorPalette() {
    assert_eq!(
        ::std::mem::size_of::<ColorPalette>(),
        12usize,
        concat!("Size of: ", stringify!(ColorPalette))
    );
    assert_eq!(
        ::std::mem::align_of::<ColorPalette>(),
        4usize,
        concat!("Alignment of ", stringify!(ColorPalette))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<ColorPalette>())).Flags as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(ColorPalette),
            "::",
            stringify!(Flags)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<ColorPalette>())).Count as *const _ as usize },
        4usize,
        concat!(
            "Offset of field: ",
            stringify!(ColorPalette),
            "::",
            stringify!(Count)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<ColorPalette>())).Entries as *const _ as usize },
        8usize,
        concat!(
            "Offset of field: ",
            stringify!(ColorPalette),
            "::",
            stringify!(Entries)
        )
    );
}
#[test]
fn bindgen_test_layout_Color() {
    assert_eq!(
        ::std::mem::size_of::<Color>(),
        4usize,
        concat!("Size of: ", stringify!(Color))
    );
    assert_eq!(
        ::std::mem::align_of::<Color>(),
        4usize,
        concat!("Alignment of ", stringify!(Color))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<Color>())).Argb as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(Color),
            "::",
            stringify!(Argb)
        )
    );
}
#[test]
fn bindgen_test_layout_ENHMETAHEADER3() {
    assert_eq!(
        ::std::mem::size_of::<ENHMETAHEADER3>(),
        88usize,
        concat!("Size of: ", stringify!(ENHMETAHEADER3))
    );
    assert_eq!(
        ::std::mem::align_of::<ENHMETAHEADER3>(),
        4usize,
        concat!("Alignment of ", stringify!(ENHMETAHEADER3))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<ENHMETAHEADER3>())).iType as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(ENHMETAHEADER3),
            "::",
            stringify!(iType)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<ENHMETAHEADER3>())).nSize as *const _ as usize },
        4usize,
        concat!(
            "Offset of field: ",
            stringify!(ENHMETAHEADER3),
            "::",
            stringify!(nSize)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<ENHMETAHEADER3>())).rclBounds as *const _ as usize },
        8usize,
        concat!(
            "Offset of field: ",
            stringify!(ENHMETAHEADER3),
            "::",
            stringify!(rclBounds)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<ENHMETAHEADER3>())).rclFrame as *const _ as usize },
        24usize,
        concat!(
            "Offset of field: ",
            stringify!(ENHMETAHEADER3),
            "::",
            stringify!(rclFrame)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<ENHMETAHEADER3>())).dSignature as *const _ as usize },
        40usize,
        concat!(
            "Offset of field: ",
            stringify!(ENHMETAHEADER3),
            "::",
            stringify!(dSignature)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<ENHMETAHEADER3>())).nVersion as *const _ as usize },
        44usize,
        concat!(
            "Offset of field: ",
            stringify!(ENHMETAHEADER3),
            "::",
            stringify!(nVersion)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<ENHMETAHEADER3>())).nBytes as *const _ as usize },
        48usize,
        concat!(
            "Offset of field: ",
            stringify!(ENHMETAHEADER3),
            "::",
            stringify!(nBytes)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<ENHMETAHEADER3>())).nRecords as *const _ as usize },
        52usize,
        concat!(
            "Offset of field: ",
            stringify!(ENHMETAHEADER3),
            "::",
            stringify!(nRecords)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<ENHMETAHEADER3>())).nHandles as *const _ as usize },
        56usize,
        concat!(
            "Offset of field: ",
            stringify!(ENHMETAHEADER3),
            "::",
            stringify!(nHandles)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<ENHMETAHEADER3>())).sReserved as *const _ as usize },
        58usize,
        concat!(
            "Offset of field: ",
            stringify!(ENHMETAHEADER3),
            "::",
            stringify!(sReserved)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<ENHMETAHEADER3>())).nDescription as *const _ as usize },
        60usize,
        concat!(
            "Offset of field: ",
            stringify!(ENHMETAHEADER3),
            "::",
            stringify!(nDescription)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<ENHMETAHEADER3>())).offDescription as *const _ as usize },
        64usize,
        concat!(
            "Offset of field: ",
            stringify!(ENHMETAHEADER3),
            "::",
            stringify!(offDescription)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<ENHMETAHEADER3>())).nPalEntries as *const _ as usize },
        68usize,
        concat!(
            "Offset of field: ",
            stringify!(ENHMETAHEADER3),
            "::",
            stringify!(nPalEntries)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<ENHMETAHEADER3>())).szlDevice as *const _ as usize },
        72usize,
        concat!(
            "Offset of field: ",
            stringify!(ENHMETAHEADER3),
            "::",
            stringify!(szlDevice)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<ENHMETAHEADER3>())).szlMillimeters as *const _ as usize },
        80usize,
        concat!(
            "Offset of field: ",
            stringify!(ENHMETAHEADER3),
            "::",
            stringify!(szlMillimeters)
        )
    );
}
#[test]
fn bindgen_test_layout_PWMFRect16() {
    assert_eq!(
        ::std::mem::size_of::<PWMFRect16>(),
        8usize,
        concat!("Size of: ", stringify!(PWMFRect16))
    );
    assert_eq!(
        ::std::mem::align_of::<PWMFRect16>(),
        2usize,
        concat!("Alignment of ", stringify!(PWMFRect16))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<PWMFRect16>())).Left as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(PWMFRect16),
            "::",
            stringify!(Left)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<PWMFRect16>())).Top as *const _ as usize },
        2usize,
        concat!(
            "Offset of field: ",
            stringify!(PWMFRect16),
            "::",
            stringify!(Top)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<PWMFRect16>())).Right as *const _ as usize },
        4usize,
        concat!(
            "Offset of field: ",
            stringify!(PWMFRect16),
            "::",
            stringify!(Right)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<PWMFRect16>())).Bottom as *const _ as usize },
        6usize,
        concat!(
            "Offset of field: ",
            stringify!(PWMFRect16),
            "::",
            stringify!(Bottom)
        )
    );
}
#[test]
fn bindgen_test_layout_WmfPlaceableFileHeader() {
    assert_eq!(
        ::std::mem::size_of::<WmfPlaceableFileHeader>(),
        22usize,
        concat!("Size of: ", stringify!(WmfPlaceableFileHeader))
    );
    assert_eq!(
        ::std::mem::align_of::<WmfPlaceableFileHeader>(),
        2usize,
        concat!("Alignment of ", stringify!(WmfPlaceableFileHeader))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<WmfPlaceableFileHeader>())).Key as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(WmfPlaceableFileHeader),
            "::",
            stringify!(Key)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<WmfPlaceableFileHeader>())).Hmf as *const _ as usize },
        4usize,
        concat!(
            "Offset of field: ",
            stringify!(WmfPlaceableFileHeader),
            "::",
            stringify!(Hmf)
        )
    );
    assert_eq!(
        unsafe {
            &(*(::std::ptr::null::<WmfPlaceableFileHeader>())).BoundingBox as *const _ as usize
        },
        6usize,
        concat!(
            "Offset of field: ",
            stringify!(WmfPlaceableFileHeader),
            "::",
            stringify!(BoundingBox)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<WmfPlaceableFileHeader>())).Inch as *const _ as usize },
        14usize,
        concat!(
            "Offset of field: ",
            stringify!(WmfPlaceableFileHeader),
            "::",
            stringify!(Inch)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<WmfPlaceableFileHeader>())).Reserved as *const _ as usize },
        16usize,
        concat!(
            "Offset of field: ",
            stringify!(WmfPlaceableFileHeader),
            "::",
            stringify!(Reserved)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<WmfPlaceableFileHeader>())).Checksum as *const _ as usize },
        20usize,
        concat!(
            "Offset of field: ",
            stringify!(WmfPlaceableFileHeader),
            "::",
            stringify!(Checksum)
        )
    );
}
#[test]
fn bindgen_test_layout_MetafileHeader__bindgen_ty_1() {
    assert_eq!(
        ::std::mem::size_of::<MetafileHeader__bindgen_ty_1>(),
        88usize,
        concat!("Size of: ", stringify!(MetafileHeader__bindgen_ty_1))
    );
    assert_eq!(
        ::std::mem::align_of::<MetafileHeader__bindgen_ty_1>(),
        4usize,
        concat!("Alignment of ", stringify!(MetafileHeader__bindgen_ty_1))
    );
    assert_eq!(
        unsafe {
            &(*(::std::ptr::null::<MetafileHeader__bindgen_ty_1>())).WmfHeader as *const _ as usize
        },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(MetafileHeader__bindgen_ty_1),
            "::",
            stringify!(WmfHeader)
        )
    );
    assert_eq!(
        unsafe {
            &(*(::std::ptr::null::<MetafileHeader__bindgen_ty_1>())).EmfHeader as *const _ as usize
        },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(MetafileHeader__bindgen_ty_1),
            "::",
            stringify!(EmfHeader)
        )
    );
}
#[test]
fn bindgen_test_layout_MetafileHeader() {
    assert_eq!(
        ::std::mem::size_of::<MetafileHeader>(),
        140usize,
        concat!("Size of: ", stringify!(MetafileHeader))
    );
    assert_eq!(
        ::std::mem::align_of::<MetafileHeader>(),
        4usize,
        concat!("Alignment of ", stringify!(MetafileHeader))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<MetafileHeader>())).Type as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(MetafileHeader),
            "::",
            stringify!(Type)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<MetafileHeader>())).Size as *const _ as usize },
        4usize,
        concat!(
            "Offset of field: ",
            stringify!(MetafileHeader),
            "::",
            stringify!(Size)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<MetafileHeader>())).Version as *const _ as usize },
        8usize,
        concat!(
            "Offset of field: ",
            stringify!(MetafileHeader),
            "::",
            stringify!(Version)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<MetafileHeader>())).EmfPlusFlags as *const _ as usize },
        12usize,
        concat!(
            "Offset of field: ",
            stringify!(MetafileHeader),
            "::",
            stringify!(EmfPlusFlags)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<MetafileHeader>())).DpiX as *const _ as usize },
        16usize,
        concat!(
            "Offset of field: ",
            stringify!(MetafileHeader),
            "::",
            stringify!(DpiX)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<MetafileHeader>())).DpiY as *const _ as usize },
        20usize,
        concat!(
            "Offset of field: ",
            stringify!(MetafileHeader),
            "::",
            stringify!(DpiY)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<MetafileHeader>())).X as *const _ as usize },
        24usize,
        concat!(
            "Offset of field: ",
            stringify!(MetafileHeader),
            "::",
            stringify!(X)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<MetafileHeader>())).Y as *const _ as usize },
        28usize,
        concat!(
            "Offset of field: ",
            stringify!(MetafileHeader),
            "::",
            stringify!(Y)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<MetafileHeader>())).Width as *const _ as usize },
        32usize,
        concat!(
            "Offset of field: ",
            stringify!(MetafileHeader),
            "::",
            stringify!(Width)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<MetafileHeader>())).Height as *const _ as usize },
        36usize,
        concat!(
            "Offset of field: ",
            stringify!(MetafileHeader),
            "::",
            stringify!(Height)
        )
    );
    assert_eq!(
        unsafe {
            &(*(::std::ptr::null::<MetafileHeader>())).EmfPlusHeaderSize as *const _ as usize
        },
        128usize,
        concat!(
            "Offset of field: ",
            stringify!(MetafileHeader),
            "::",
            stringify!(EmfPlusHeaderSize)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<MetafileHeader>())).LogicalDpiX as *const _ as usize },
        132usize,
        concat!(
            "Offset of field: ",
            stringify!(MetafileHeader),
            "::",
            stringify!(LogicalDpiX)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<MetafileHeader>())).LogicalDpiY as *const _ as usize },
        136usize,
        concat!(
            "Offset of field: ",
            stringify!(MetafileHeader),
            "::",
            stringify!(LogicalDpiY)
        )
    );
}
#[test]
fn bindgen_test_layout_ImageCodecInfo() {
    assert_eq!(
        ::std::mem::size_of::<ImageCodecInfo>(),
        104usize,
        concat!("Size of: ", stringify!(ImageCodecInfo))
    );
    assert_eq!(
        ::std::mem::align_of::<ImageCodecInfo>(),
        8usize,
        concat!("Alignment of ", stringify!(ImageCodecInfo))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<ImageCodecInfo>())).Clsid as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(ImageCodecInfo),
            "::",
            stringify!(Clsid)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<ImageCodecInfo>())).FormatID as *const _ as usize },
        16usize,
        concat!(
            "Offset of field: ",
            stringify!(ImageCodecInfo),
            "::",
            stringify!(FormatID)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<ImageCodecInfo>())).CodecName as *const _ as usize },
        32usize,
        concat!(
            "Offset of field: ",
            stringify!(ImageCodecInfo),
            "::",
            stringify!(CodecName)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<ImageCodecInfo>())).DllName as *const _ as usize },
        40usize,
        concat!(
            "Offset of field: ",
            stringify!(ImageCodecInfo),
            "::",
            stringify!(DllName)
        )
    );
    assert_eq!(
        unsafe {
            &(*(::std::ptr::null::<ImageCodecInfo>())).FormatDescription as *const _ as usize
        },
        48usize,
        concat!(
            "Offset of field: ",
            stringify!(ImageCodecInfo),
            "::",
            stringify!(FormatDescription)
        )
    );
    assert_eq!(
        unsafe {
            &(*(::std::ptr::null::<ImageCodecInfo>())).FilenameExtension as *const _ as usize
        },
        56usize,
        concat!(
            "Offset of field: ",
            stringify!(ImageCodecInfo),
            "::",
            stringify!(FilenameExtension)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<ImageCodecInfo>())).MimeType as *const _ as usize },
        64usize,
        concat!(
            "Offset of field: ",
            stringify!(ImageCodecInfo),
            "::",
            stringify!(MimeType)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<ImageCodecInfo>())).Flags as *const _ as usize },
        72usize,
        concat!(
            "Offset of field: ",
            stringify!(ImageCodecInfo),
            "::",
            stringify!(Flags)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<ImageCodecInfo>())).Version as *const _ as usize },
        76usize,
        concat!(
            "Offset of field: ",
            stringify!(ImageCodecInfo),
            "::",
            stringify!(Version)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<ImageCodecInfo>())).SigCount as *const _ as usize },
        80usize,
        concat!(
            "Offset of field: ",
            stringify!(ImageCodecInfo),
            "::",
            stringify!(SigCount)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<ImageCodecInfo>())).SigSize as *const _ as usize },
        84usize,
        concat!(
            "Offset of field: ",
            stringify!(ImageCodecInfo),
            "::",
            stringify!(SigSize)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<ImageCodecInfo>())).SigPattern as *const _ as usize },
        88usize,
        concat!(
            "Offset of field: ",
            stringify!(ImageCodecInfo),
            "::",
            stringify!(SigPattern)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<ImageCodecInfo>())).SigMask as *const _ as usize },
        96usize,
        concat!(
            "Offset of field: ",
            stringify!(ImageCodecInfo),
            "::",
            stringify!(SigMask)
        )
    );
}
#[test]
fn bindgen_test_layout_BitmapData() {
    assert_eq!(
        ::std::mem::size_of::<BitmapData>(),
        32usize,
        concat!("Size of: ", stringify!(BitmapData))
    );
    assert_eq!(
        ::std::mem::align_of::<BitmapData>(),
        8usize,
        concat!("Alignment of ", stringify!(BitmapData))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<BitmapData>())).Width as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(BitmapData),
            "::",
            stringify!(Width)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<BitmapData>())).Height as *const _ as usize },
        4usize,
        concat!(
            "Offset of field: ",
            stringify!(BitmapData),
            "::",
            stringify!(Height)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<BitmapData>())).Stride as *const _ as usize },
        8usize,
        concat!(
            "Offset of field: ",
            stringify!(BitmapData),
            "::",
            stringify!(Stride)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<BitmapData>())).PixelFormat as *const _ as usize },
        12usize,
        concat!(
            "Offset of field: ",
            stringify!(BitmapData),
            "::",
            stringify!(PixelFormat)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<BitmapData>())).Scan0 as *const _ as usize },
        16usize,
        concat!(
            "Offset of field: ",
            stringify!(BitmapData),
            "::",
            stringify!(Scan0)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<BitmapData>())).Reserved as *const _ as usize },
        24usize,
        concat!(
            "Offset of field: ",
            stringify!(BitmapData),
            "::",
            stringify!(Reserved)
        )
    );
}
#[test]
fn bindgen_test_layout_EncoderParameter() {
    assert_eq!(
        ::std::mem::size_of::<EncoderParameter>(),
        32usize,
        concat!("Size of: ", stringify!(EncoderParameter))
    );
    assert_eq!(
        ::std::mem::align_of::<EncoderParameter>(),
        8usize,
        concat!("Alignment of ", stringify!(EncoderParameter))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<EncoderParameter>())).Guid as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(EncoderParameter),
            "::",
            stringify!(Guid)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<EncoderParameter>())).NumberOfValues as *const _ as usize },
        16usize,
        concat!(
            "Offset of field: ",
            stringify!(EncoderParameter),
            "::",
            stringify!(NumberOfValues)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<EncoderParameter>())).Type as *const _ as usize },
        20usize,
        concat!(
            "Offset of field: ",
            stringify!(EncoderParameter),
            "::",
            stringify!(Type)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<EncoderParameter>())).Value as *const _ as usize },
        24usize,
        concat!(
            "Offset of field: ",
            stringify!(EncoderParameter),
            "::",
            stringify!(Value)
        )
    );
}
#[test]
fn bindgen_test_layout_EncoderParameters() {
    assert_eq!(
        ::std::mem::size_of::<EncoderParameters>(),
        40usize,
        concat!("Size of: ", stringify!(EncoderParameters))
    );
    assert_eq!(
        ::std::mem::align_of::<EncoderParameters>(),
        8usize,
        concat!("Alignment of ", stringify!(EncoderParameters))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<EncoderParameters>())).Count as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(EncoderParameters),
            "::",
            stringify!(Count)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<EncoderParameters>())).Parameter as *const _ as usize },
        8usize,
        concat!(
            "Offset of field: ",
            stringify!(EncoderParameters),
            "::",
            stringify!(Parameter)
        )
    );
}
#[test]
fn bindgen_test_layout_PropertyItem() {
    assert_eq!(
        ::std::mem::size_of::<PropertyItem>(),
        24usize,
        concat!("Size of: ", stringify!(PropertyItem))
    );
    assert_eq!(
        ::std::mem::align_of::<PropertyItem>(),
        8usize,
        concat!("Alignment of ", stringify!(PropertyItem))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<PropertyItem>())).id as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(PropertyItem),
            "::",
            stringify!(id)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<PropertyItem>())).length as *const _ as usize },
        4usize,
        concat!(
            "Offset of field: ",
            stringify!(PropertyItem),
            "::",
            stringify!(length)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<PropertyItem>())).type_ as *const _ as usize },
        8usize,
        concat!(
            "Offset of field: ",
            stringify!(PropertyItem),
            "::",
            stringify!(type_)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<PropertyItem>())).value as *const _ as usize },
        16usize,
        concat!(
            "Offset of field: ",
            stringify!(PropertyItem),
            "::",
            stringify!(value)
        )
    );
}
#[test]
fn bindgen_test_layout_ColorMatrix() {
    assert_eq!(
        ::std::mem::size_of::<ColorMatrix>(),
        100usize,
        concat!("Size of: ", stringify!(ColorMatrix))
    );
    assert_eq!(
        ::std::mem::align_of::<ColorMatrix>(),
        4usize,
        concat!("Alignment of ", stringify!(ColorMatrix))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<ColorMatrix>())).m as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(ColorMatrix),
            "::",
            stringify!(m)
        )
    );
}
#[test]
fn bindgen_test_layout_ColorMap() {
    assert_eq!(
        ::std::mem::size_of::<ColorMap>(),
        8usize,
        concat!("Size of: ", stringify!(ColorMap))
    );
    assert_eq!(
        ::std::mem::align_of::<ColorMap>(),
        4usize,
        concat!("Alignment of ", stringify!(ColorMap))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<ColorMap>())).oldColor as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(ColorMap),
            "::",
            stringify!(oldColor)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<ColorMap>())).newColor as *const _ as usize },
        4usize,
        concat!(
            "Offset of field: ",
            stringify!(ColorMap),
            "::",
            stringify!(newColor)
        )
    );
}
#[test]
fn bindgen_test_layout_GpGraphics() {
    assert_eq!(
        ::std::mem::size_of::<GpGraphics>(),
        1usize,
        concat!("Size of: ", stringify!(GpGraphics))
    );
    assert_eq!(
        ::std::mem::align_of::<GpGraphics>(),
        1usize,
        concat!("Alignment of ", stringify!(GpGraphics))
    );
}
#[test]
fn bindgen_test_layout_GpBrush() {
    assert_eq!(
        ::std::mem::size_of::<GpBrush>(),
        1usize,
        concat!("Size of: ", stringify!(GpBrush))
    );
    assert_eq!(
        ::std::mem::align_of::<GpBrush>(),
        1usize,
        concat!("Alignment of ", stringify!(GpBrush))
    );
}
#[test]
fn bindgen_test_layout_GpTexture() {
    assert_eq!(
        ::std::mem::size_of::<GpTexture>(),
        1usize,
        concat!("Size of: ", stringify!(GpTexture))
    );
    assert_eq!(
        ::std::mem::align_of::<GpTexture>(),
        1usize,
        concat!("Alignment of ", stringify!(GpTexture))
    );
}
#[test]
fn bindgen_test_layout_GpSolidFill() {
    assert_eq!(
        ::std::mem::size_of::<GpSolidFill>(),
        1usize,
        concat!("Size of: ", stringify!(GpSolidFill))
    );
    assert_eq!(
        ::std::mem::align_of::<GpSolidFill>(),
        1usize,
        concat!("Alignment of ", stringify!(GpSolidFill))
    );
}
#[test]
fn bindgen_test_layout_GpLineGradient() {
    assert_eq!(
        ::std::mem::size_of::<GpLineGradient>(),
        1usize,
        concat!("Size of: ", stringify!(GpLineGradient))
    );
    assert_eq!(
        ::std::mem::align_of::<GpLineGradient>(),
        1usize,
        concat!("Alignment of ", stringify!(GpLineGradient))
    );
}
#[test]
fn bindgen_test_layout_GpPathGradient() {
    assert_eq!(
        ::std::mem::size_of::<GpPathGradient>(),
        1usize,
        concat!("Size of: ", stringify!(GpPathGradient))
    );
    assert_eq!(
        ::std::mem::align_of::<GpPathGradient>(),
        1usize,
        concat!("Alignment of ", stringify!(GpPathGradient))
    );
}
#[test]
fn bindgen_test_layout_GpHatch() {
    assert_eq!(
        ::std::mem::size_of::<GpHatch>(),
        1usize,
        concat!("Size of: ", stringify!(GpHatch))
    );
    assert_eq!(
        ::std::mem::align_of::<GpHatch>(),
        1usize,
        concat!("Alignment of ", stringify!(GpHatch))
    );
}
#[test]
fn bindgen_test_layout_GpPen() {
    assert_eq!(
        ::std::mem::size_of::<GpPen>(),
        1usize,
        concat!("Size of: ", stringify!(GpPen))
    );
    assert_eq!(
        ::std::mem::align_of::<GpPen>(),
        1usize,
        concat!("Alignment of ", stringify!(GpPen))
    );
}
#[test]
fn bindgen_test_layout_GpCustomLineCap() {
    assert_eq!(
        ::std::mem::size_of::<GpCustomLineCap>(),
        1usize,
        concat!("Size of: ", stringify!(GpCustomLineCap))
    );
    assert_eq!(
        ::std::mem::align_of::<GpCustomLineCap>(),
        1usize,
        concat!("Alignment of ", stringify!(GpCustomLineCap))
    );
}
#[test]
fn bindgen_test_layout_GpAdjustableArrowCap() {
    assert_eq!(
        ::std::mem::size_of::<GpAdjustableArrowCap>(),
        1usize,
        concat!("Size of: ", stringify!(GpAdjustableArrowCap))
    );
    assert_eq!(
        ::std::mem::align_of::<GpAdjustableArrowCap>(),
        1usize,
        concat!("Alignment of ", stringify!(GpAdjustableArrowCap))
    );
}
#[test]
fn bindgen_test_layout_GpImage() {
    assert_eq!(
        ::std::mem::size_of::<GpImage>(),
        1usize,
        concat!("Size of: ", stringify!(GpImage))
    );
    assert_eq!(
        ::std::mem::align_of::<GpImage>(),
        1usize,
        concat!("Alignment of ", stringify!(GpImage))
    );
}
#[test]
fn bindgen_test_layout_GpBitmap() {
    assert_eq!(
        ::std::mem::size_of::<GpBitmap>(),
        1usize,
        concat!("Size of: ", stringify!(GpBitmap))
    );
    assert_eq!(
        ::std::mem::align_of::<GpBitmap>(),
        1usize,
        concat!("Alignment of ", stringify!(GpBitmap))
    );
}
#[test]
fn bindgen_test_layout_GpMetafile() {
    assert_eq!(
        ::std::mem::size_of::<GpMetafile>(),
        1usize,
        concat!("Size of: ", stringify!(GpMetafile))
    );
    assert_eq!(
        ::std::mem::align_of::<GpMetafile>(),
        1usize,
        concat!("Alignment of ", stringify!(GpMetafile))
    );
}
#[test]
fn bindgen_test_layout_GpImageAttributes() {
    assert_eq!(
        ::std::mem::size_of::<GpImageAttributes>(),
        1usize,
        concat!("Size of: ", stringify!(GpImageAttributes))
    );
    assert_eq!(
        ::std::mem::align_of::<GpImageAttributes>(),
        1usize,
        concat!("Alignment of ", stringify!(GpImageAttributes))
    );
}
#[test]
fn bindgen_test_layout_GpPath() {
    assert_eq!(
        ::std::mem::size_of::<GpPath>(),
        1usize,
        concat!("Size of: ", stringify!(GpPath))
    );
    assert_eq!(
        ::std::mem::align_of::<GpPath>(),
        1usize,
        concat!("Alignment of ", stringify!(GpPath))
    );
}
#[test]
fn bindgen_test_layout_GpRegion() {
    assert_eq!(
        ::std::mem::size_of::<GpRegion>(),
        1usize,
        concat!("Size of: ", stringify!(GpRegion))
    );
    assert_eq!(
        ::std::mem::align_of::<GpRegion>(),
        1usize,
        concat!("Alignment of ", stringify!(GpRegion))
    );
}
#[test]
fn bindgen_test_layout_GpPathIterator() {
    assert_eq!(
        ::std::mem::size_of::<GpPathIterator>(),
        1usize,
        concat!("Size of: ", stringify!(GpPathIterator))
    );
    assert_eq!(
        ::std::mem::align_of::<GpPathIterator>(),
        1usize,
        concat!("Alignment of ", stringify!(GpPathIterator))
    );
}
#[test]
fn bindgen_test_layout_GpFontFamily() {
    assert_eq!(
        ::std::mem::size_of::<GpFontFamily>(),
        1usize,
        concat!("Size of: ", stringify!(GpFontFamily))
    );
    assert_eq!(
        ::std::mem::align_of::<GpFontFamily>(),
        1usize,
        concat!("Alignment of ", stringify!(GpFontFamily))
    );
}
#[test]
fn bindgen_test_layout_GpFont() {
    assert_eq!(
        ::std::mem::size_of::<GpFont>(),
        1usize,
        concat!("Size of: ", stringify!(GpFont))
    );
    assert_eq!(
        ::std::mem::align_of::<GpFont>(),
        1usize,
        concat!("Alignment of ", stringify!(GpFont))
    );
}
#[test]
fn bindgen_test_layout_GpStringFormat() {
    assert_eq!(
        ::std::mem::size_of::<GpStringFormat>(),
        1usize,
        concat!("Size of: ", stringify!(GpStringFormat))
    );
    assert_eq!(
        ::std::mem::align_of::<GpStringFormat>(),
        1usize,
        concat!("Alignment of ", stringify!(GpStringFormat))
    );
}
#[test]
fn bindgen_test_layout_GpFontCollection() {
    assert_eq!(
        ::std::mem::size_of::<GpFontCollection>(),
        1usize,
        concat!("Size of: ", stringify!(GpFontCollection))
    );
    assert_eq!(
        ::std::mem::align_of::<GpFontCollection>(),
        1usize,
        concat!("Alignment of ", stringify!(GpFontCollection))
    );
}
#[test]
fn bindgen_test_layout_Region() {
    assert_eq!(
        ::std::mem::size_of::<Region>(),
        16usize,
        concat!("Size of: ", stringify!(Region))
    );
    assert_eq!(
        ::std::mem::align_of::<Region>(),
        8usize,
        concat!("Alignment of ", stringify!(Region))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<Region>())).nativeRegion as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(Region),
            "::",
            stringify!(nativeRegion)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<Region>())).lastResult as *const _ as usize },
        8usize,
        concat!(
            "Offset of field: ",
            stringify!(Region),
            "::",
            stringify!(lastResult)
        )
    );
}
#[test]
fn bindgen_test_layout_FontFamily() {
    assert_eq!(
        ::std::mem::size_of::<FontFamily>(),
        16usize,
        concat!("Size of: ", stringify!(FontFamily))
    );
    assert_eq!(
        ::std::mem::align_of::<FontFamily>(),
        8usize,
        concat!("Alignment of ", stringify!(FontFamily))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<FontFamily>())).nativeFamily as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(FontFamily),
            "::",
            stringify!(nativeFamily)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<FontFamily>())).lastResult as *const _ as usize },
        8usize,
        concat!(
            "Offset of field: ",
            stringify!(FontFamily),
            "::",
            stringify!(lastResult)
        )
    );
}
#[test]
fn bindgen_test_layout_Font() {
    assert_eq!(
        ::std::mem::size_of::<Font>(),
        16usize,
        concat!("Size of: ", stringify!(Font))
    );
    assert_eq!(
        ::std::mem::align_of::<Font>(),
        8usize,
        concat!("Alignment of ", stringify!(Font))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<Font>())).nativeFont as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(Font),
            "::",
            stringify!(nativeFont)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<Font>())).lastResult as *const _ as usize },
        8usize,
        concat!(
            "Offset of field: ",
            stringify!(Font),
            "::",
            stringify!(lastResult)
        )
    );
}
#[test]
fn bindgen_test_layout_FontCollection() {
    assert_eq!(
        ::std::mem::size_of::<FontCollection>(),
        24usize,
        concat!("Size of: ", stringify!(FontCollection))
    );
    assert_eq!(
        ::std::mem::align_of::<FontCollection>(),
        8usize,
        concat!("Alignment of ", stringify!(FontCollection))
    );
    assert_eq!(
        unsafe {
            &(*(::std::ptr::null::<FontCollection>())).nativeFontCollection as *const _ as usize
        },
        8usize,
        concat!(
            "Offset of field: ",
            stringify!(FontCollection),
            "::",
            stringify!(nativeFontCollection)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<FontCollection>())).lastResult as *const _ as usize },
        16usize,
        concat!(
            "Offset of field: ",
            stringify!(FontCollection),
            "::",
            stringify!(lastResult)
        )
    );
}
#[test]
fn bindgen_test_layout_InstalledFontCollection() {
    assert_eq!(
        ::std::mem::size_of::<InstalledFontCollection>(),
        24usize,
        concat!("Size of: ", stringify!(InstalledFontCollection))
    );
    assert_eq!(
        ::std::mem::align_of::<InstalledFontCollection>(),
        8usize,
        concat!("Alignment of ", stringify!(InstalledFontCollection))
    );
}
#[test]
fn bindgen_test_layout_PrivateFontCollection() {
    assert_eq!(
        ::std::mem::size_of::<PrivateFontCollection>(),
        24usize,
        concat!("Size of: ", stringify!(PrivateFontCollection))
    );
    assert_eq!(
        ::std::mem::align_of::<PrivateFontCollection>(),
        8usize,
        concat!("Alignment of ", stringify!(PrivateFontCollection))
    );
}
#[test]
fn bindgen_test_layout_Image() {
    assert_eq!(
        ::std::mem::size_of::<Image>(),
        24usize,
        concat!("Size of: ", stringify!(Image))
    );
    assert_eq!(
        ::std::mem::align_of::<Image>(),
        8usize,
        concat!("Alignment of ", stringify!(Image))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<Image>())).nativeImage as *const _ as usize },
        8usize,
        concat!(
            "Offset of field: ",
            stringify!(Image),
            "::",
            stringify!(nativeImage)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<Image>())).lastResult as *const _ as usize },
        16usize,
        concat!(
            "Offset of field: ",
            stringify!(Image),
            "::",
            stringify!(lastResult)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<Image>())).loadStatus as *const _ as usize },
        20usize,
        concat!(
            "Offset of field: ",
            stringify!(Image),
            "::",
            stringify!(loadStatus)
        )
    );
}
#[test]
fn bindgen_test_layout_Bitmap() {
    assert_eq!(
        ::std::mem::size_of::<Bitmap>(),
        24usize,
        concat!("Size of: ", stringify!(Bitmap))
    );
    assert_eq!(
        ::std::mem::align_of::<Bitmap>(),
        8usize,
        concat!("Alignment of ", stringify!(Bitmap))
    );
}
#[test]
fn bindgen_test_layout_CustomLineCap() {
    assert_eq!(
        ::std::mem::size_of::<CustomLineCap>(),
        24usize,
        concat!("Size of: ", stringify!(CustomLineCap))
    );
    assert_eq!(
        ::std::mem::align_of::<CustomLineCap>(),
        8usize,
        concat!("Alignment of ", stringify!(CustomLineCap))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<CustomLineCap>())).nativeCap as *const _ as usize },
        8usize,
        concat!(
            "Offset of field: ",
            stringify!(CustomLineCap),
            "::",
            stringify!(nativeCap)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<CustomLineCap>())).lastResult as *const _ as usize },
        16usize,
        concat!(
            "Offset of field: ",
            stringify!(CustomLineCap),
            "::",
            stringify!(lastResult)
        )
    );
}
#[test]
fn bindgen_test_layout_CachedBitmap() {
    assert_eq!(
        ::std::mem::size_of::<CachedBitmap>(),
        24usize,
        concat!("Size of: ", stringify!(CachedBitmap))
    );
    assert_eq!(
        ::std::mem::align_of::<CachedBitmap>(),
        8usize,
        concat!("Alignment of ", stringify!(CachedBitmap))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<CachedBitmap>())).nativeCachedBitmap as *const _ as usize },
        8usize,
        concat!(
            "Offset of field: ",
            stringify!(CachedBitmap),
            "::",
            stringify!(nativeCachedBitmap)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<CachedBitmap>())).lastResult as *const _ as usize },
        16usize,
        concat!(
            "Offset of field: ",
            stringify!(CachedBitmap),
            "::",
            stringify!(lastResult)
        )
    );
}
#[test]
fn bindgen_test_layout_Metafile() {
    assert_eq!(
        ::std::mem::size_of::<Metafile>(),
        24usize,
        concat!("Size of: ", stringify!(Metafile))
    );
    assert_eq!(
        ::std::mem::align_of::<Metafile>(),
        8usize,
        concat!("Alignment of ", stringify!(Metafile))
    );
}
#[test]
fn bindgen_test_layout_Matrix() {
    assert_eq!(
        ::std::mem::size_of::<Matrix>(),
        16usize,
        concat!("Size of: ", stringify!(Matrix))
    );
    assert_eq!(
        ::std::mem::align_of::<Matrix>(),
        8usize,
        concat!("Alignment of ", stringify!(Matrix))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<Matrix>())).nativeMatrix as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(Matrix),
            "::",
            stringify!(nativeMatrix)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<Matrix>())).lastResult as *const _ as usize },
        8usize,
        concat!(
            "Offset of field: ",
            stringify!(Matrix),
            "::",
            stringify!(lastResult)
        )
    );
}
#[test]
fn bindgen_test_layout_Pen() {
    assert_eq!(
        ::std::mem::size_of::<Pen>(),
        16usize,
        concat!("Size of: ", stringify!(Pen))
    );
    assert_eq!(
        ::std::mem::align_of::<Pen>(),
        8usize,
        concat!("Alignment of ", stringify!(Pen))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<Pen>())).nativePen as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(Pen),
            "::",
            stringify!(nativePen)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<Pen>())).lastResult as *const _ as usize },
        8usize,
        concat!(
            "Offset of field: ",
            stringify!(Pen),
            "::",
            stringify!(lastResult)
        )
    );
}
#[test]
fn bindgen_test_layout_StringFormat() {
    assert_eq!(
        ::std::mem::size_of::<StringFormat>(),
        16usize,
        concat!("Size of: ", stringify!(StringFormat))
    );
    assert_eq!(
        ::std::mem::align_of::<StringFormat>(),
        8usize,
        concat!("Alignment of ", stringify!(StringFormat))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<StringFormat>())).nativeFormat as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(StringFormat),
            "::",
            stringify!(nativeFormat)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<StringFormat>())).lastError as *const _ as usize },
        8usize,
        concat!(
            "Offset of field: ",
            stringify!(StringFormat),
            "::",
            stringify!(lastError)
        )
    );
}
#[test]
fn bindgen_test_layout_GraphicsPath() {
    assert_eq!(
        ::std::mem::size_of::<GraphicsPath>(),
        16usize,
        concat!("Size of: ", stringify!(GraphicsPath))
    );
    assert_eq!(
        ::std::mem::align_of::<GraphicsPath>(),
        8usize,
        concat!("Alignment of ", stringify!(GraphicsPath))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<GraphicsPath>())).nativePath as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(GraphicsPath),
            "::",
            stringify!(nativePath)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<GraphicsPath>())).lastResult as *const _ as usize },
        8usize,
        concat!(
            "Offset of field: ",
            stringify!(GraphicsPath),
            "::",
            stringify!(lastResult)
        )
    );
}
#[test]
fn bindgen_test_layout_Graphics() {
    assert_eq!(
        ::std::mem::size_of::<Graphics>(),
        16usize,
        concat!("Size of: ", stringify!(Graphics))
    );
    assert_eq!(
        ::std::mem::align_of::<Graphics>(),
        8usize,
        concat!("Alignment of ", stringify!(Graphics))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<Graphics>())).nativeGraphics as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(Graphics),
            "::",
            stringify!(nativeGraphics)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<Graphics>())).lastResult as *const _ as usize },
        8usize,
        concat!(
            "Offset of field: ",
            stringify!(Graphics),
            "::",
            stringify!(lastResult)
        )
    );
}
