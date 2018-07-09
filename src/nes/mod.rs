pub mod cpu;
use self::cpu::Tick;

pub mod mapper;
pub mod memory;
//mod ines;
pub mod ppu;

use std::cell::RefCell;
use std::fs::File;
use std::rc::Rc;

pub struct Nes<M>
where
    M: mapper::Mapper,
{
    cpu: cpu::Cpu<M>,
    mem: Rc<RefCell<memory::Memory<M>>>,
    ppu: Rc<RefCell<ppu::Ppu<M>>>,
    int_bus: Rc<RefCell<InterruptBus>>,
}

impl<M> Nes<M>
where
    M: mapper::Mapper,
{
    pub fn new() -> Nes<mapper::Nrom> {
        //let header = ines::parse_header(&mut File::open("donkey kong.nes").unwrap());
        let mapper = Rc::new(RefCell::new(mapper::Nrom::new()));
        Nes::<mapper::Nrom>::new_private(mapper)
    }

    fn new_private<T>(mapper: Rc<RefCell<T>>) -> Nes<T>
    where
        T: mapper::Mapper,
    {
        let int_bus = Rc::new(RefCell::new(InterruptBus::new()));

        let ppu = Rc::new(RefCell::new(ppu::Ppu::new(int_bus.clone(), mapper.clone())));
        let mem = Rc::new(RefCell::new(memory::Memory::new(
            ppu.clone(),
            mapper.clone(),
        )));
        mem.borrow_mut().load_mapper_0(
            &mut File::open("src/tests/blargg_instr/rom_singles/01-basics.nes").unwrap(),
        );

        /* mem.borrow_mut().load_mapper_0_1(
            &mut File::open("donkey kong.nes").unwrap(),
        ); */

        let mut cpu = cpu::Cpu::new(mem.clone(), int_bus.clone());
        cpu.gen_reset();

        Nes {
            cpu,
            mem,
            ppu,
            int_bus,
        }
    }

    pub fn run(&mut self) {
        let mut ppu_cycle = 0;
        self.cpu.gen_reset();

        while !self.cpu.halt {
            ppu_cycle += 1;
            //self.cpu.print_debug_info();
            if self.mem.borrow().dma_cycles > 0 {
                self.mem.borrow_mut().dma_cycles -= 1;
            } else {
                self.cpu.tick();
            }

            self.ppu.borrow_mut().tick();
            self.ppu.borrow_mut().tick();
            self.ppu.borrow_mut().tick();

            /* let mut s: String = String::new();
            stdin.read_line(&mut s); */
        }
    }
}

#[derive(Clone, Copy)]
pub struct InterruptBus {
    pub irq_signal: bool,
    pub nmi_signal: bool,
    pub reset_signal: bool,
}

impl InterruptBus {
    pub fn new() -> InterruptBus {
        InterruptBus {
            irq_signal: false,
            nmi_signal: false,
            reset_signal: false,
        }
    }
}
