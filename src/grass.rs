#[path = "./tool.rs"]
mod tool;

use piston_window::*;
use cgmath::*;
use std::{f64::consts::PI};

use crate::{pid_line::*, settings::Settings};
use tool::*;
use crate::wind_particle::*;

pub struct Grass
{
    pub pid_lines           : Vec<PidLine>,
    part_lengths            : Vec<f64>,
    part_radie              : Vec<f64>,

    pub total_line_length   : u64, 
    pub line_amount         : usize, 
    pub ratio               : f64,
    pub position            : Vector2<f64>,
    pub radius              : f64,
    pub angle               : f64,
    pub pid                 : PID,
    bend_factor             : f64,
    base_line_radius        : f64,
}

impl Grass
{
    pub fn new(
            total_line_length   : u64, 
            line_amount         : usize, 
            ratio               : f64,
            position            : Vector2<f64>,
            radius              : f64,
            pid                 : PID
        )-> Grass
    {
        Grass
        {
            pid_lines           : Vec::new(),
            part_lengths        : vec![0f64; line_amount],
            
            part_radie          : vec![0f64; line_amount],

            total_line_length   : total_line_length,
            base_line_radius    : radius,
            line_amount         : line_amount,
            ratio               : ratio,
            position            : position,
            radius              : radius,
            angle               : 0f64,
            pid                 : pid,
            bend_factor         : 0_f64,
        }
    }

    #[allow(dead_code)]
    fn calc_lengths(
            total_line_length: u64, 
            line_amount: usize, 
            ratio : f64
        ) -> Vec<f64>
    {
        let mut l: Vec<f64> = Vec::new();

        let mut sum: f64 = 0f64;
        for i in (0..line_amount).rev()
        {
            let value: f64 = ratio.powf(i as f64 + 1f64);
            l.push(value);

            sum += value;
        }

        let r: f64 = total_line_length as f64/sum;
        for i in 0..line_amount
        {
            l[i] *= r;
        }

        l
    }

    #[allow(dead_code)]
    fn calc_radie(
            base_line_radius: f64, 
            line_amount: usize, 
            ratio : f64
        ) -> Vec<f64>
    {
        let mut l: Vec<f64> = Vec::new();

        for i in 0..line_amount
        {
            let value: f64 = base_line_radius / ratio.powf(i as f64 + 1f64);
            l.push(value);
        }

        l
    }

    pub fn draw(&mut self, c : Context, g : &mut G2d)
    {
        for i in 0..self.line_amount
        {
            self.pid_lines[i].draw(c, g);
        }
    }

    pub fn init(&mut self)
    {
        // self.part_lengths = vec![128f64,64f64,32f64,16f64, 8f64];
        self.part_lengths = Grass::calc_lengths(
                                        self.total_line_length,
                                        self.line_amount,
                                        self.ratio
                                    );

        self.part_radie = Grass::calc_radie(
                                        self.base_line_radius,
                                        self.line_amount,
                                        self.ratio
                                    );
        for i in 0..self.line_amount
        {
            self.pid_lines.push(PidLine::new(
                                        Vector2::new(320.0, 400.0), 
                                        self.angle, 
                                        self.part_lengths[i], 
                                        self.part_radie[i],
                                        [0.0, 0.5+i as f32/10f32, 0.0, 1.0],
                                        self.pid 
                                    )
                                );
        }
    }

    pub fn update(&mut self, u: UpdateArgs)
    {   
        self.pid_lines[0].length    = self.part_lengths[0];
        self.pid_lines[0].radius    = self.radius;
        self.pid_lines[0].position  = self.position;
        self.pid_lines[0].update(u);

        for i in 1..self.line_amount
        {
            /*set the length and thiccness of the parts. */
            self.pid_lines[i].length    = self.part_lengths[i];
            self.pid_lines[i].radius    = self.part_radie[i];
            // self.pid_lines[i].angle     = self.pid_lines[i-1].angle + 
            //                                 self.pid_lines[0].angle/self.ratio * self.bend_factor;            

            self.pid_lines[i].position  = self.pid_lines[i-1].end_point;

            let end: Vector2<f64> = Tool::new_vec2_with_angle(self.pid_lines[i].length, -self.pid_lines[i].angle + PI/2f64);
            let start: Vector2<f64> = self.pid_lines[i].position;
            self.pid_lines[i].end_point = Vector2::new(end.x + start.x, start.y- end.y);

            self.pid_lines[i].update(u);
        }   
    }   

    pub fn update_wind(&mut self, u: UpdateArgs, w: &WindParticle, s: &Settings)
    {
        if (w.position.x - self.position.x).abs() > s.get_range_values("wind", "react_horizontal")[1]
        {
            return;
        }

        if (w.position.y - self.position.y).abs() > s.get_range_values("wind", "react_vertical")[1]
        {
            return;
        }

        for i in 0..self.line_amount
        {
            self.pid_lines[i].pid.integral += Tool::normalize_between(
                                                        (w.position.x - self.position.x).abs(), 
                                                        s.get_range_values("wind", "react_horizontal")[0], 
                                                        s.get_range_values("wind", "react_horizontal")[1], 
                                                        s.get_range_values("wind", "integral")[0], 
                                                        s.get_range_values("wind", "integral")[1]
                                                    ) * u.dt * w.speed * i as f64;       
        }

        self.bend_factor = Tool::normalize_between(
                                self.position.distance(w.position), 
                                0_f64, 
                                self.total_line_length as f64, 
                                s.get_range_values("wind", "bend")[0], 
                                s.get_range_values("wind", "bend")[1]
                            ) + w.direction * w.speed;
    }
}