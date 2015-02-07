use std::ffi::CString;
use std::old_io::File;


// TODO: IDs shouldn't be copyable, and they should have a parameter for what
// type they're an ID for; ideally, IDs for different types shouldn't be
// comparable
pub type Id = u64;

pub struct IdGen {
  next: Id,
}

impl IdGen {
  pub fn new() -> IdGen {IdGen {next: 0} }
  pub fn next(&mut self) -> Id {
    let id = self.next;
    self.next += 1;
    id
  }
}


// TODO: this is a terrible hack
static mut global_id_gen: IdGen = IdGen {next: 0};

pub fn next_id() -> Id {
  unsafe {global_id_gen.next()}
}



// TODO: get rid of this
pub fn c_str_from_slice(string: &str) -> *const i8 {
  let c_string = CString::from_slice(string.as_bytes());
  c_string.as_ptr()
}


pub fn read_file(path: &Path) -> String {
  let contents = match File::open(path).read_to_end().ok() {
    Some(contents) => contents,
    None => panic!("Failed to read file {}", path.display()),
  };
  String::from_utf8(contents).ok().expect("Failed to read file")
}
