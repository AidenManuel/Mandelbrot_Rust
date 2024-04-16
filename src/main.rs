extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;
extern crate rand;
extern crate chrono;

use glutin_window::GlutinWindow as Window;
use num::integer::sqrt;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderArgs, RenderEvent, UpdateArgs, UpdateEvent};
use piston::window::WindowSettings;
use piston::GenericEvent;
use rayon::prelude::*;
use std::sync::atomic::{AtomicI32, Ordering};
use std::thread::ThreadId;

use num::complex::Complex as cmp;

const GRAPH_SCALE: f64 = 100.0;
const ITERATIONS: i16 = 1200;

const MAGIC_RE: f64 = 0.3602404434376143632361252444495453084826;
const MAGIC_IM: f64 = -0.641313061064803174860375015179302066579;

const RE1: f64 = MAGIC_RE - 2.0;
const RE2: f64 = MAGIC_RE + 2.0;
const DRE: f64 = RE2 - RE1;

const IM1: f64 = MAGIC_IM - 1.0;
const IM2: f64 = MAGIC_IM + 1.0;
const DIM: f64 = IM2 - IM1;

const RAT: f64 = DIM / DRE;

const RE_MIN: i16 = (RE1 * GRAPH_SCALE) as i16;
const RE_MAX: i16 = (RE2 * GRAPH_SCALE) as i16;
const DOMAIN: usize = (RE_MAX - RE_MIN) as usize;

const IM_MIN: i16 = (IM1 * GRAPH_SCALE) as i16;
const IM_MAX: i16 = (IM2 * GRAPH_SCALE) as i16;
const RANGE: usize = (IM_MAX - IM_MIN) as usize;

pub struct App { 
    // OpenGL drawing backend.
    gl: GlGraphics,
    vals: [[i16; DOMAIN]; RANGE],
    re_min: f64,
    re_max: f64,
    im_min: f64,
    im_max: f64,
    re_scale: f64,
    im_scale: f64,
    zoom: f64,
    scalar: f32,
    step_factor: f32,
    paused: bool,
}

impl App {
    /////////
    /// RENDER FUNCTION
    ///////

    fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        // Defining all necessary constants

        let black: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
        let mut colour = black;
        let mut colour_mod = 0.0;

        /////////
        // DRAW FUNCTION
        ///////

        for b in 0..RANGE {
            for a in 0..DOMAIN {

                // We draw each cell as a square, which is a data structure
                // with 4 floating point values.
                let square = rectangle::square(a as f64, b as f64, 1.0);
                
                // OpenGL is used for rendering it to the screen.
                self.gl.draw(args.viewport(), |c, gl| {

                    if self.vals[b][a] == ITERATIONS {
                        colour = black;
                    } else {
                        if self.scalar > 0.05 {
                            colour_mod = self.vals[b][a] as f32 / (100 as f32) as f32 * self.scalar; 
                        } else {

                            colour_mod = self.vals[b][a] as f32 / (100 as f64) as f32 * 0.05; 
                        }
                        
                    
                        colour = [colour_mod * 2.4, colour_mod * 2.0, colour_mod * 3.0, 1.0];
                    }

                    let transform = c
                        .transform;

                    rectangle(colour, square, transform, gl);
                });
            }
        }
    }

    
    /////////
    /// UPDATE FUNCTION
    //////
    
    // PARALLEL UPDATE CODE
    
    fn update_parallel(&mut self, _args: &UpdateArgs) {
        if !self.paused {
            let bound = cmp::new(2.0, 0.0);
            const MIDDLE_IM: f64 = RANGE as f64 / 2.0;
            const MIDDLE_RE: f64 = DOMAIN as f64 / 2.0;

            let mut values: [[i16; DOMAIN]; RANGE] = [[0; DOMAIN]; RANGE];

            values.par_iter_mut()
                .enumerate()
                .for_each(|(im, b)| {

                    let mut z: cmp<f64>;
                    let mut z_next: cmp<f64>;
                    let mut c: cmp<f64>;
                    let mut done = false;
                    let mut count = 0;
                    let mut a_float: f64;
                    let mut b_float: f64;

                    for a in 0..DOMAIN {
                        //let center_dist = f64::sqrt(f64::powf(MIDDLE_IM - im as f64, 2.0) + f64::powf(MIDDLE_RE - a as f64, 2.0));
                        //println!("d = {0}, a = {1}, b = {2}", center_dist, a, im);
                        (a_float, b_float) = ((a as f64 / self.re_scale + self.re_min), (im as f64 / self.im_scale + self.im_min));
                        c = cmp::new(a_float, b_float);
                        z = cmp::new(0.0, 0.0);
                        
                        while !done && count < (ITERATIONS) {
                            z_next = z * z + c;
                            z = z_next;
                            count += 1;
                    
                            if cmp::norm_sqr(&z) >= cmp::norm_sqr(&bound) {
                                done = true;
                            }
                        }

                        b[a] += count;

                        done = false;
                        count = 0;
                    }
                });

            self.vals = values;

            let re_zoom = self.zoom;
            let im_zoom = re_zoom * RAT;

            let re_scalar = (self.re_max - self.re_min) / (self.re_max - self.re_min - (2.0 * re_zoom));
            let im_scalar = (self.im_max - self.im_min) / (self.im_max - self.im_min - (2.0 * im_zoom));

            self.re_min += re_zoom;
            self.re_max -= re_zoom;
            self.im_min += im_zoom;
            self.im_max -= im_zoom;

            self.re_scale *= re_scalar;
            self.im_scale *= im_scalar;
            
            self.zoom *= 0.95;

            if self.scalar > 0.000005 {
                self.step_factor = 0.000001;
            }
            if self.scalar > 0.00005 {
                self.step_factor = 0.00001;
            }
            if self.scalar > 0.0005 {
                self.step_factor = 0.0001;
            }
            if self.scalar > 0.01 {
                self.step_factor = 0.001;
            }
            if self.scalar > 0.23 {
                self.step_factor = 0.01
            }

            self.scalar -= self.step_factor;
        }
        
    }

    // SEQUENTIAL UPDATE CODE

    fn update_sequential(&mut self, _args: &UpdateArgs) {
        if !self.paused {
            let bound = cmp::new(2.0, 0.0);

            let mut z: cmp<f64>;
            let mut z_next: cmp<f64>;
            let mut c: cmp<f64>;
            let mut done = false;
            let mut count = 0;
            let mut a_float: f64;
            let mut b_float: f64;

            for a in 0..DOMAIN {
                for b in 0..RANGE {
                    (a_float, b_float) = ((a as f64 / self.re_scale + self.re_min), (b as f64 / self.im_scale + self.im_min));
                    c = cmp::new(a_float, b_float);
                    z = cmp::new(0.0, 0.0);
                    
                    while !done && count < ITERATIONS {
                        z_next = z * z + c;
                        z = z_next;
                        count += 1;
                
                        if cmp::norm_sqr(&z) >= cmp::norm_sqr(&bound) {
                            done = true;
                        }
                    }
                    self.vals[b][a] = count;

                    done = false;
                    count = 0;
                }
            }


            let re_zoom = self.zoom;
            let im_zoom = re_zoom * RAT;

            let re_scalar = (self.re_max - self.re_min) / (self.re_max - self.re_min - (2.0 * re_zoom));
            let im_scalar = (self.im_max - self.im_min) / (self.im_max - self.im_min - (2.0 * im_zoom));

            self.re_min += re_zoom;
            self.re_max -= re_zoom;
            self.im_min += im_zoom;
            self.im_max -= im_zoom;

            self.re_scale *= re_scalar;
            self.im_scale *= im_scalar;
            
            self.zoom *= 0.95;

            if self.scalar > 0.000005 {
                self.step_factor = 0.000001;
            }
            if self.scalar > 0.00005 {
                self.step_factor = 0.00001;
            }
            if self.scalar > 0.0005 {
                self.step_factor = 0.0001;
            }
            if self.scalar > 0.01 {
                self.step_factor = 0.001;
            }
            if self.scalar > 0.23 {
                self.step_factor = 0.01
            }

            self.scalar -= self.step_factor;
        }
    }

    fn event<E: GenericEvent>(&mut self, pos: [f64; 2], e: &E) {
        use piston::input::{Button, Key};

        // Key Functions Added!
        // Space:   pause the simulation
        // P:       print the current information
        if let Some(Button::Keyboard(key)) = e.press_args() {
                let mut i = 0;
                match key {
                    Key::Space => {self.paused = !self.paused; if self.paused { println!("paused") } else { println!("playing") };},
                    Key::P => self.print(),
                    _ => {}
            }
        }
    }

    fn print(&mut self) {
        println!(">===---\nre_min={0}\nre_max={1}\nim_min={2}\nim_max={3}\nre_scale={4}\nim_scale={5}\nzoom={6}\nscalar={7}\nstep_factor={8}\nGRAPH_SCALE={9}\n>===---", 
                 self.re_min, self.re_max, self.im_min, self.im_max, self.re_scale, self.im_scale, self.zoom, self.scalar, self.step_factor, GRAPH_SCALE);
    }

}

///////////////////////////////
// Most of this main method was not programmed by me. It comes
// from a Piston tutorial, which is the crate I'm using to update
// the do the graphics. See example from repo here:
// https://github.com/PistonDevelopers/Piston-Tutorials/tree/master/getting-started
///////////////////////////////

fn main() {
    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;

    // Create a Glutin window.
    let mut window: Window = WindowSettings::new("Mandelbrot", [DOMAIN as f64, RANGE as f64])
        .graphics_api(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    // Create a new game and run it.

    let vals = [[0; DOMAIN]; RANGE];

    let mut app = App {
        gl: GlGraphics::new(opengl),
        vals: vals,
        re_min: RE1,
        re_max: RE2,
        im_min: IM1,
        im_max: IM2,
        re_scale: GRAPH_SCALE,
        im_scale: GRAPH_SCALE,
        zoom: 0.10,
        scalar: 2.0,
        step_factor:0.01,
        paused: false,
    };

    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut window) {
        app.event([0.0, 0.0], &e);

        if let Some(args) = e.render_args() {
            app.render(&args);
        }

        if let Some(args) = e.update_args() {
            app.update_parallel(&args);
        }
    }
}
