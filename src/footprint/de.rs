// (c) 2016 Productize SPRL <joost@productize.be>

use Sexp;
use str_error;
use Result;
use footprint::data::*;

pub trait FromSexp {
    fn from_sexp(&Sexp) -> Self;
}

// (at 0.0 -4.0) (at -2.575 -1.625 180)
impl FromSexp for Result<At> {
    fn from_sexp(s:&Sexp) -> Result<At> {
        let v = try!(s.slice_atom("at"));
        match v.len() {
            2 => {
                let x = try!(v[0].f());
                let y = try!(v[1].f());
                Ok(At::new(x, y, 0.0))
            }
            3 => {
                let x = try!(v[0].f());
                let y = try!(v[1].f());
                let rot = try!(v[2].f());
                Ok(At::new(x, y, rot))
            }
            _ => str_error("at with wrong length".to_string())
        }
    }
}

pub fn wrap<X,Y,F,G>(s:&Sexp, make:F, wrapper:G) -> Result<Y>
    where F:Fn(&Sexp) -> Result<X>, G:Fn(X) -> Y
{
    Ok(wrapper(try!(make(s))))
}


impl FromSexp for Result<Layer> {
    fn from_sexp(s:&Sexp) -> Result<Layer> {
        let v = try!(s.slice_atom("layer"));
        let layer = try!(v[0].string());
        let layer = try!(Layer::from_string(layer.clone()));
        Ok(layer)
    }
}

impl FromSexp for Result<Effects> {
    fn from_sexp(s:&Sexp) -> Result<Effects> {
        //let v = try!(s.slice_atom_num("effects", 1));
        // TODO investigate why the above doesn't work !?
        let v = try!(s.slice_atom("effects"));
        if v.len() < 1 {
            return str_error(format!("Expected at least one element in {}", s))
        }
        let font = try!(Result::from_sexp(&v[0]));
        let justify = if v.len() > 1 {
            Some(try!(Result::from_sexp(&v[1])))
        } else {
            None
        };
        Ok(Effects::from_font(font, justify))
    }
}

impl FromSexp for Result<Justify> {
    fn from_sexp(s:&Sexp) -> Result<Justify> {
        let v = try!(s.slice_atom("justify"));
        if v.len() < 1 {
            return str_error(format!("Expected at least one element in {}", s))
        }
        let s = try!(v[0].string());
        match &s[..] {
            "mirror" => Ok(Justify::Mirror),
            _ => str_error(format!("unknown justify: {}", s))
        }
    }
}

impl FromSexp for Result<Font> {
    fn from_sexp(s:&Sexp) -> Result<Font> {
        let v = try!(s.slice_atom("font"));
        let parts = try!(parse_parts(&v));
        let mut font = Font::default();
        for part in &parts[..] {
            //println!("part: {}", part);
            try!(match *part {
                Part::Xy(ref xy) if xy.t == XyType::Size => {
                    font.size.x = xy.x;
                    font.size.y = xy.y;
                    Ok(())
                }
                Part::Thickness(ref t) => {
                    font.thickness = *t;
                    Ok(())
                }
                ref x => Err(format!("unknown element in font: {:?}", x))
            })
        }
        Ok(font)
    }
}

impl FromSexp for Result<Layers> {
    fn from_sexp(s:&Sexp) -> Result<Layers> {
        let mut l = Layers::default();
        let v = try!(s.slice_atom("layers"));
        for v1 in v {
            let x = try!(v1.string());
            let layer = try!(Layer::from_string(x.clone()));
            l.append(&layer)
        }
        Ok(l)
    }
}

fn parse_part_float<F>(e: &Sexp, make:F) -> Result<Part>
    where F:Fn(f64) -> Part
{
    let v = try!(e.list());
    if v.len() < 2 {
        return str_error(format!("not enough elements in {}", e))
    }
    let f = try!(v[1].f());
    Ok(make(f))
}

impl FromSexp for Result<Vec<Xy> > {
    fn from_sexp(s:&Sexp) -> Result<Vec<Xy> > {
        let v = try!(s.slice_atom("pts"));
        let mut pts = vec![];
        for e in &v[1..] {
            let p = try!(Result::from_sexp(e));
            pts.push(p)
        }
        Ok(pts)
    }
}

impl FromSexp for Result<Xy> {
    fn from_sexp(s: &Sexp) -> Result<Xy> {
        let name = try!(s.list_name());
        let t = try!(match &name[..] {
            "xy" => Ok(XyType::Xy),
            "start" => Ok(XyType::Start),
            "end" => Ok(XyType::End),
            "size" => Ok(XyType::Size),
            "center" => Ok(XyType::Center),
            "rect_delta" => Ok(XyType::RectDelta),
            ref x => str_error(format!("unknown XyType {}", x)),
        });
        let v = try!(s.slice_atom_num(&name, 2));
        let x = try!(v[0].f());
        let y = try!(v[1].f());
        Ok(Xy::new(x,y,t))
    }
}

impl FromSexp for Result<Pts> {
    fn from_sexp(s: &Sexp) -> Result<Pts> {
        let v = try!(s.slice_atom("pts"));
        let mut r = vec![];
        for x in v {
            let xy = try!(Result::from_sexp(x));
            r.push(xy)
        }
        Ok(Pts { elements:r })
    }
}


impl FromSexp for Result<Xyz> {
    fn from_sexp(s: &Sexp) -> Result<Xyz> {
        let v = try!(s.slice_atom_num("xyz", 3));
        let x = try!(v[0].f());
        let y = try!(v[1].f());
        let z = try!(v[2].f());
        Ok(Xyz::new(x,y,z))
    }
}

impl FromSexp for Result<Net> {
    fn from_sexp(s: &Sexp) -> Result<Net> {
        let v = try!(s.slice_atom_num("net", 2));
        let num = try!(v[0].i());
        let name = try!(v[1].string());
        Ok(Net { num:num, name:name.clone(), })
    }
}

impl FromSexp for Result<Drill> {
    fn from_sexp(s: &Sexp) -> Result<Drill> {
        let v = try!(s.slice_atom("drill"));
        if v.len() == 1 {
            let drill = try!(v[0].f());
            Ok(Drill { shape:None, drill:drill, drill2:None })
                
        } else if v.len() == 3 {
            let shape = try!(v[0].string());
            let drill = try!(v[1].f());
            let drill2 = try!(v[2].f());
            Ok(Drill { shape:Some(shape.clone()), drill:drill, drill2:Some(drill2) })
        } else {
            str_error("unknown drill format".to_string())
        }
    }
}

impl FromSexp for Result<Part> {
    fn from_sexp(s:&Sexp) -> Result<Part> {
        match s.string() {
            Ok(ref sx) => match &sx[..] {
                "hide" => Ok(Part::Hide),
                x => str_error(format!("unknown part in element: {}", x))
            },
            _ => {
                let name = &try!(s.list_name())[..];
                match name {
                    "at" => wrap(s, Result::from_sexp, Part::At),
                    "layer" => wrap(s, Result::from_sexp, Part::Layer),
                    "effects" => wrap(s, Result::from_sexp, Part::Effects),
                    "layers" => wrap(s, Result::from_sexp, Part::Layers),
                    "width" => parse_part_float(s, Part::Width),
                    "angle" => parse_part_float(s, Part::Angle),
                    "start" | "end" | "size" | "center" | "rect_delta" => wrap(s, Result::from_sexp, Part::Xy),
                    "pts" => wrap(s, Result::from_sexp, Part::Pts),
                    "thickness" => parse_part_float(s, Part::Thickness),
                    "net" => wrap(s, Result::from_sexp, Part::Net),
                    "drill" => wrap(s, Result::from_sexp, Part::Drill),
                    "solder_paste_margin" => parse_part_float(s, Part::SolderPasteMargin),
                    "solder_mask_margin" => parse_part_float(s, Part::SolderMaskMargin),
                    "clearance" => parse_part_float(s, Part::Clearance),
                    x => str_error(format!("unknown part {}", x))
                }
            }
        }
    }        
}

fn parse_parts(v: &[Sexp]) -> Result<Vec<Part>> {
    let mut res = Vec::new();
    for e in v {
        let p = try!(Result::from_sexp(e));
        res.push(p);
    }
    Ok(res)
}

fn parse_string_element(s:&Sexp) -> Result<String> {
    let name = try!(s.list_name());
    let v = try!(s.slice_atom_num(&name, 1));
    let s = try!(v[0].string());
    Ok(s.clone())
}

fn parse_float_element(s:&Sexp) -> Result<f64> {
    let name = try!(s.list_name());
    let v = try!(s.slice_atom_num(&name, 1));
    let f = try!(v[0].f());
    Ok(f)
}

impl FromSexp for Result<FpText> {
    fn from_sexp(s:&Sexp) -> Result<FpText> {
        let v = try!(s.slice_atom("fp_text"));
        let name = try!(v[0].string());
        let value = try!(v[1].string());
        let parts = try!(parse_parts(&v[2..]));
        let mut fp = FpText::new(name.clone(), value.clone());
        for part in &parts[..] {
            match *part {
                Part::At(ref at) => {
                    fp.at.clone_from(at)
                },
                Part::Layer(ref layer) => {
                    fp.set_layer(layer)
                }
                Part::Hide => {
                    fp.hide = true
                },
                Part::Effects(ref effects) => {
                    fp.set_effects(effects)
                }
                ref x => {
                    return str_error(format!("fp_text: unknown {:?}", x))
                }
            }
        }
        Ok(fp)
    }
}

impl FromSexp for Result<Pad> {
    fn from_sexp(s:&Sexp) -> Result<Pad> {
        let v = try!(s.slice_atom("pad"));
        if v.len() < 3 {
            return str_error("not enough elements in pad".to_string())
        }
        let name = try!(v[0].string()).clone();
        let t = try!(v[1].string());
        let t = try!(PadType::from_string(&t));
        let shape = try!(v[2].string());
        let shape = try!(PadShape::from_string(&shape));
        let mut pad = Pad::new(name, t, shape);
        //println!("{}", pad);
        let parts = try!(parse_parts(&v[3..]));
        for part in &parts[..] {
            match *part {
                Part::At(ref at) => pad.at.clone_from(at),
                Part::Xy(ref xy) if xy.t == XyType::Size => pad.size.clone_from(xy),
                Part::Xy(ref xy) if xy.t == XyType::RectDelta => pad.rect_delta = Some(xy.clone()),
                Part::Layers(ref l) => pad.layers.clone_from(l),
                Part::Net(ref n) => pad.set_net(n),
                Part::Drill(ref n) => pad.set_drill(n),
                Part::SolderPasteMargin(n) => pad.solder_paste_margin = Some(n),
                Part::SolderMaskMargin(n) => pad.solder_mask_margin = Some(n),
                Part::Clearance(n) => pad.clearance = Some(n),
                ref x => return str_error(format!("pad: unknown {:?}", x)),
            }
        }
        Ok(pad)
    }
}

impl FromSexp for Result<FpPoly> {
    fn from_sexp(s:&Sexp) -> Result<FpPoly> {
        let v = try!(s.slice_atom("fp_poly"));
        let mut fp_poly = FpPoly::default();
        let parts = try!(parse_parts(&v));
        for part in &parts[..] {
            match *part {
                Part::Pts(ref pts) => fp_poly.pts.clone_from(pts),
                Part::Width(w) => fp_poly.width = w,
                Part::Layer(ref layer) => fp_poly.layer.clone_from(layer),
                ref x => println!("fp_poly: ignoring {:?}", x),
            }
        } 
        Ok(fp_poly)
    }
}

impl FromSexp for Result<FpLine> {
    fn from_sexp(s:&Sexp) -> Result<FpLine> {
        let v = try!(s.slice_atom("fp_line"));
        let mut fp_line = FpLine::default();
        let parts = try!(parse_parts(&v));
        for part in &parts[..] {
            match *part {
                Part::Xy(ref xy) if xy.t == XyType::Start => fp_line.start.clone_from(xy),
                Part::Xy(ref xy) if xy.t == XyType::End => fp_line.end.clone_from(xy),
                Part::Layer(ref layer) => fp_line.layer.clone_from(layer),
                Part::Width(w) => fp_line.width = w,
                ref x => return str_error(format!("fp_line: unknown {:?}", x)),
            }
        }
        Ok(fp_line)
    }
}

impl FromSexp for Result<FpCircle> {
    fn from_sexp(s:&Sexp) -> Result<FpCircle> {
        let v = try!(s.slice_atom("fp_circle"));
        let mut fp_circle = FpCircle::default();
        let parts = try!(parse_parts(&v));
        for part in &parts[..] {
            match *part {
                Part::Xy(ref xy) if xy.t == XyType::Center => fp_circle.center.clone_from(xy),
                Part::Xy(ref xy) if xy.t == XyType::End => fp_circle.end.clone_from(xy),
                Part::Layer(ref layer) => fp_circle.layer.clone_from(layer),
                Part::Width(w) => fp_circle.width = w,
                ref x => return str_error(format!("fp_circle: unexpected {:?}", x)),
            }
        }
        Ok(fp_circle)
    }
}

impl FromSexp for Result<FpArc> {
    fn from_sexp(s:&Sexp) -> Result<FpArc> {
        let v = try!(s.slice_atom("fp_arc"));
        let mut fp_arc = FpArc::default();
        let parts = try!(parse_parts(&v));
        for part in &parts[..] {
            match *part {
                Part::Xy(ref xy) if xy.t == XyType::Start => fp_arc.start.clone_from(xy),
                Part::Xy(ref xy) if xy.t == XyType::End => fp_arc.end.clone_from(xy),
                Part::Angle(w) => fp_arc.angle = w,
                Part::Layer(ref layer) => fp_arc.layer.clone_from(layer),
                Part::Width(w) => fp_arc.width = w,
                ref x => return str_error(format!("fp_arc: unexpected {:?}", x)),
            }
        }
        Ok(fp_arc)
    }
}


fn parse_sublist<X>(s:&Sexp, name:&'static str) -> Result<X>
    where Result<X>:FromSexp {
    let x = &try!(s.slice_atom_num(name, 1))[0];
    Result::from_sexp(x)
}


impl FromSexp for Result<Model> {
    fn from_sexp(s:&Sexp) -> Result<Model> {
        let v = try!(s.slice_atom_num("model", 4));
        let name = try!(v[0].string()).clone();
        let at = try!(parse_sublist(&v[1], "at"));
        let scale = try!(parse_sublist(&v[2], "scale"));
        let rotate = try!(parse_sublist(&v[3], "rotate"));
        let m = Model {name:name, at:at, scale:scale, rotate:rotate};
        Ok(m)
    }
}

impl FromSexp for Result<Element> {
    fn from_sexp(s:&Sexp) -> Result<Element> {
        match *s {
            Sexp::String(ref s) => {
                match &s[..] {
                    "locked" => Ok(Element::Locked),
                    _ => str_error(format!("unknown element in module: {}", s))
                }
            },
            Sexp::List(_) => {
                let name = try!(s.list_name());
                match &name[..] {
                    "solder_mask_margin" => wrap(s, parse_float_element, Element::SolderMaskMargin),
                    "layer" => wrap(s, parse_string_element, Element::Layer),
                    "descr" => wrap(s, parse_string_element, Element::Descr),
                    "tags" => wrap(s, parse_string_element, Element::Tags),
                    "attr" => wrap(s, parse_string_element, Element::Attr),
                    "fp_text" => wrap(s, Result::from_sexp, Element::FpText),
                    "pad" => wrap(s, Result::from_sexp, Element::Pad),
                    "fp_poly" => wrap(s, Result::from_sexp, Element::FpPoly),
                    "fp_line" => wrap(s, Result::from_sexp, Element::FpLine),
                    "fp_circle" => wrap(s, Result::from_sexp, Element::FpCircle),
                    "fp_arc" => wrap(s, Result::from_sexp, Element::FpArc),
                    "tedit" => wrap(s, parse_string_element, Element::TEdit),
                    "tstamp" => wrap(s, parse_string_element, Element::TStamp),
                    "path" => wrap(s, parse_string_element, Element::Path),
                    "at" => wrap(s, Result::from_sexp, Element::At),
                    "model" => wrap(s, Result::from_sexp, Element::Model),
                    _ => str_error(format!("unknown element in module: {}", name))
                }
            },
            Sexp::Empty => unreachable!(),
        }
    }
}

impl FromSexp for Result<Module> {
    fn from_sexp(s:&Sexp) -> Result<Module> {
        let v = try!(s.slice_atom("module"));
        if v.len() < 1 {
            return str_error("no name in module".to_string());
        }
        let name = try!(v[0].string());
        let mut module = Module::new(name.clone());
        for e in &v[1..] {
            let el = try!(Result::from_sexp(&e));
            module.append(el)
        }
        Ok(module)
    }
}

