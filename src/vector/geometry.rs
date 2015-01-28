use std::ptr::null;
use libc::{c_char, c_int, c_double};
use std::ffi::CString;
use utils::_string;
use vector::{ogr, geom};
use GdalError;


pub struct Geometry {
    c_geometry: *const (),
}


impl Geometry {
    pub fn empty(wkb_type: c_int) -> Geometry {
        let c_geom = unsafe { ogr::OGR_G_CreateGeometry(wkb_type) };
        assert!(c_geom != null());
        return Geometry{c_geometry: c_geom};
    }

    pub fn from_wkt(wkt: &str) -> Geometry {
        let c_wkt = CString::from_slice(wkt.as_bytes());
        let mut c_wkt_ptr: *const c_char = c_wkt.as_ptr();
        let mut c_geom: *const () = null();
        let rv = unsafe { ogr::OGR_G_CreateFromWkt(&mut c_wkt_ptr, null(), &mut c_geom) };
        assert_eq!(rv, ogr::OGRERR_NONE);
        return Geometry{c_geometry: c_geom};
    }

    pub fn bbox(w: f64, s: f64, e: f64, n: f64) -> Geometry {
        Geometry::from_wkt(format!(
            "POLYGON (({} {}, {} {}, {} {}, {} {}, {} {}))",
            w, n,
            e, n,
            e, s,
            w, s,
            w, n,
        ).as_slice())
    }

    pub fn json(&self) -> String {
        let c_json = unsafe { ogr::OGR_G_ExportToJson(self.c_geometry) };
        let rv = _string(c_json);
        unsafe { ogr::VSIFree(c_json as *mut ()) };
        return rv;
    }

    pub fn wkt(&self) -> String {
        let mut c_wkt: *const c_char = null();
        let _err = unsafe { ogr::OGR_G_ExportToWkt(self.c_geometry, &mut c_wkt) };
        assert_eq!(_err, ogr::OGRERR_NONE);
        let wkt = _string(c_wkt);
        unsafe { ogr::OGRFree(c_wkt as *mut ()) };
        return wkt;
    }

    pub unsafe fn c_geometry(&self) -> *const () {
        return self.c_geometry;
    }

    pub fn set_point_2d(&mut self, i: isize, p: (f64, f64)) {
        let (x, y) = p;
        unsafe { ogr::OGR_G_SetPoint_2D(
            self.c_geometry,
            i as c_int,
            x as c_double,
            y as c_double,
        ) };
    }

    pub fn get_point(&self, i: isize) -> (f64, f64, f64) {
        let mut x: c_double = 0.;
        let mut y: c_double = 0.;
        let mut z: c_double = 0.;
        unsafe { ogr::OGR_G_GetPoint(self.c_geometry, 0, &mut x, &mut y, &mut z) };
        return (x as f64, y as f64, z as f64);
    }

    pub fn to_geom(&self) -> Result<geom::Geom, GdalError> {
        let geometry_type = unsafe { ogr::OGR_G_GetGeometryType(self.c_geometry) };
        match geometry_type {
            1 => {
                let (x, y, _) = self.get_point(0);
                Ok(geom::Geom::Point(geom::Point{x: x, y: y}))
            },
            _ => Err(GdalError{desc: "Unknown geometry type"})
        }
    }
}


impl Drop for Geometry {
    fn drop(&mut self) {
        unsafe { ogr::OGR_G_DestroyGeometry(self.c_geometry as *mut ()) };
    }
}


pub trait ToGdal {
    fn to_gdal(&self) -> Geometry;
}


impl ToGdal for geom::Point {
    fn to_gdal(&self) -> Geometry {
        let mut geom = Geometry::empty(ogr::wkbPoint);
        geom.set_point_2d(0, (self.x, self.y));
        return geom;
    }
}


#[cfg(test)]
mod tests {
    use vector::{Geometry, ToGdal};
    use vector::geom::{Geom, Point};

    #[test]
    fn test_ogr_to_point() {
        let g = Geometry::from_wkt("POINT (10 20)".as_slice());
        assert_eq!(g.to_geom(), Ok(Geom::Point(Point{x: 10., y: 20.})));
    }

    #[test]
    fn test_point_to_ogr() {
        let g = Point{x: 10., y: 20.}.to_gdal();
        assert_eq!(g.wkt(), "POINT (10 20)");
    }
}