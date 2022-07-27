#![no_main]
use rust_vmm_fuzz::FuzzingDescriptor;
use vm_memory::{GuestAddress, GuestMemoryMmap};
use virtio_queue::{mock::MockSplitQueue, Descriptor};
use virtio_vsock::packet::VsockPacket;
use libfuzzer_sys::{fuzz_target, arbitrary::Arbitrary};

/// All the functions that can be called on a VsockPacket
#[derive(Arbitrary, Debug)]
pub enum VsockFunctionType<'a> {
    HeaderSlice,
    Len,
    DataSlice,
    SrcCid,
    SetSrcCid { cid: u64 },
    DstCid,
    SetDstCid { cid: u64 },
    SrcPort,
    SetSrcPort { port: u32 },
    DstPort,
    SetDstPort { port: u32 },
    IsEmpty,
    SetLen { len: u32 },
    Type_,
    SetType { type_: u16 },
    Op,
    SetOp { op: u16 },
    Flags,
    SetFlags { flags: u32 },
    SetFlag { flag: u32 },
    BufAlloc,
    SetBufAlloc { buf_alloc: u32 },
    FwdCnt,
    SetFwdCnt { fwd_cnt: u32 },
    SetHeaderFromRaw { bytes: &'a [u8] },
}

impl VsockFunctionType<'_> {
    pub fn call<B: vm_memory::bitmap::BitmapSlice>(&self, packet: &mut VsockPacket<B>) {
        match self {
            VsockFunctionType::HeaderSlice => { packet.header_slice(); },
            VsockFunctionType::Len => { packet.len(); },
            VsockFunctionType::DataSlice => { packet.data_slice(); },
            VsockFunctionType::SrcCid => { packet.src_cid(); },
            VsockFunctionType::SetSrcCid { cid } => { packet.set_src_cid(*cid); },
            VsockFunctionType::DstCid => { packet.dst_cid(); },
            VsockFunctionType::SetDstCid { cid } => { packet.set_dst_cid(*cid); },
            VsockFunctionType::SrcPort => { packet.src_port(); },
            VsockFunctionType::SetSrcPort { port } => { packet.set_src_port(*port); },
            VsockFunctionType::DstPort => { packet.dst_port(); },
            VsockFunctionType::SetDstPort { port } => { packet.set_dst_port(*port); },
            VsockFunctionType::IsEmpty => { packet.is_empty(); },
            VsockFunctionType::SetLen { len } => { packet.set_len(*len); },
            VsockFunctionType::Type_ => { packet.type_(); },
            VsockFunctionType::SetType { type_ } => { packet.set_type(*type_); },
            VsockFunctionType::Op => { packet.op(); },
            VsockFunctionType::SetOp { op } => { packet.set_op(*op); },
            VsockFunctionType::Flags => { packet.flags(); },
            VsockFunctionType::SetFlags { flags } => { packet.set_flags(*flags); },
            VsockFunctionType::SetFlag { flag } => { packet.set_flag(*flag); },
            VsockFunctionType::BufAlloc => { packet.buf_alloc(); },
            VsockFunctionType::SetBufAlloc { buf_alloc } => { packet.set_buf_alloc(*buf_alloc); },
            VsockFunctionType::FwdCnt => { packet.fwd_cnt(); },
            VsockFunctionType::SetFwdCnt { fwd_cnt } => { packet.set_fwd_cnt(*fwd_cnt); },
            VsockFunctionType::SetHeaderFromRaw { bytes } => { let _ = packet.set_header_from_raw(*bytes); },
        }
    }
}

// Whether we create a VsockPacket from_rx_virtq_chain or from_tx_virtq_chain
#[derive(Arbitrary, Debug, Copy, Clone)]
pub enum InitFunction {
    FromRX,
    FromTX,
}

/// Input generated by the fuzzer for fuzzing vsock_rx and vsock_tx
#[derive(Arbitrary, Debug)]
pub struct VsockInput<'a> {
    pub pkt_max_data: u32,
    pub init_function: InitFunction,
    pub descriptors: Vec<FuzzingDescriptor>,
    pub functions: Vec<VsockFunctionType<'a>>,
}

fuzz_target!(|fuzz_input: VsockInput| {
    let m = &GuestMemoryMmap::<()>::from_ranges(&[(GuestAddress(0), 0x10000)]).unwrap();
    let vq = MockSplitQueue::new(m, fuzz_input.descriptors.len() as u16);

    let descriptors: Vec<Descriptor> = fuzz_input.descriptors.iter().map(|desc| (*desc).into()).collect();

    if let Ok(mut chain) = vq.build_desc_chain(&descriptors) {
        let packet = match fuzz_input.init_function {
            InitFunction::FromRX => {
                VsockPacket::from_rx_virtq_chain(m, &mut chain, fuzz_input.pkt_max_data)
            },
            InitFunction::FromTX => {
                VsockPacket::from_tx_virtq_chain(m, &mut chain, fuzz_input.pkt_max_data)
            },
        };
        if let Ok(mut p) = packet {
            fuzz_input.functions.iter().for_each(|f| f.call(&mut p));
        }
    }
});
