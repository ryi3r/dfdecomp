#![feature(anonymous_lifetime_in_impl_trait)]
#![feature(strict_provenance)]

use std::{net::TcpStream, io::{Cursor, Seek, SeekFrom, BufWriter, Write}, collections::HashMap, fs::File};
use byteorder::{LittleEndian, ReadBytesExt};
use tracing::{info, error, warn};

#[ctor::ctor]
fn ctor() {
    std::thread::spawn(|| {
        let stream = TcpStream::connect("127.0.0.1:7331").unwrap();
        color_eyre::install().unwrap();
        tracing_subscriber::fmt()
            .with_writer(std::sync::Mutex::new(stream))
            .init();

        info!("Hello from the DLL (Injected on DF)!!");

        std::thread::sleep(std::time::Duration::from_secs_f64(0.5));
        let result = std::thread::spawn(|| {
            unsafe { do_fallible_stuff() }
        }).join();
        if let Err(e) = result {
            error!("Error: {:?}", e.downcast_ref::<String>().unwrap());
        }
    });
}

unsafe fn do_fallible_stuff() -> color_eyre::Result<()> {
    // Let's start searching for the data.win
    // We expect Bytecode 16, since DF uses GMS 1.4.9999.

    // On DF 2.7.6c, there's 5 pointers to the data.win
    // 0x47a4d1, 0x47b2ad, 0x83dd7c, 0x83e00c and 0x83e098
    // I'll use the first one for now.

    // How do find the pointer:
    // 0xa4e5e0 contains the Game ID, we need to search
    // all u32 addresses pointing to 0xa4e5e0, the 2nd
    // one should be our data.win, which the data.win
    // would offset to pointer - 0x24.

    unsafe {

        // We're now going into the main loop, where handling pointers is unsafe,
        // we need to be careful of not crashing DF by accident because of a
        // wrong pointer, meaning a complete data loss.

        let data_pointer = *(0x47a4d1 as *mut u32) as *mut u32;
        let data_length = *data_pointer.offset(1) as usize;
        let mut data = Cursor::new(Vec::from_raw_parts(data_pointer.addr() as *mut u8, data_length, data_length));

        macro_rules! read_string {
            () => {
                {
                    let pointer = data.read_u32::<LittleEndian>().unwrap();
                    let position = data.position();
                    data.seek(SeekFrom::Start(pointer as u64)).unwrap();
                    let mut string = Vec::new();
                    let mut buffer = data.read_u8().unwrap();
                    while buffer != 0 {
                        string.push(buffer);
                        buffer = data.read_u8().unwrap();
                    }
                    let string = String::from_utf8_lossy(&string).to_string();
                    data.seek(SeekFrom::Start(position)).unwrap();

                    string
                }
            };
        }
        macro_rules! read_u8 {
            () => {
                {
                    data.read_u8().unwrap()
                }
            };
        }
        macro_rules! read_u16 {
            () => {
                {
                    data.read_u16::<LittleEndian>().unwrap()
                }
            };
        }
        macro_rules! read_u32 {
            () => {
                {
                    data.read_u32::<LittleEndian>().unwrap()
                }
            };
        }
        macro_rules! read_u64 {
            () => {
                {
                    data.read_u64::<LittleEndian>().unwrap()
                }
            };
        }
        #[allow(unused_macros)]
        macro_rules! read_i8 {
            () => {
                {
                    data.read_i8().unwrap()
                }
            };
        }
        macro_rules! read_i16 {
            () => {
                {
                    data.read_i16::<LittleEndian>().unwrap()
                }
            };
        }
        macro_rules! read_i32 {
            () => {
                {
                    data.read_i32::<LittleEndian>().unwrap()
                }
            };
        }
        #[allow(unused_macros)]
        macro_rules! read_i64 {
            () => {
                {
                    data.read_i64::<LittleEndian>().unwrap()
                }
            };
        }
        macro_rules! read_f32 {
            () => {
                {
                    data.read_f32::<LittleEndian>().unwrap()
                }
            };
        }
        #[allow(unused_macros)]
        macro_rules! read_f64 {
            () => {
                {
                    data.read_f64::<LittleEndian>().unwrap()
                }
            };
        }
        macro_rules! read_bytes {
            ($size: expr) => {
                {
                    let mut d = [0u8; $size];
                    for i in 0..$size {
                        d[i] = data.read_u8().unwrap();
                    }
                    d
                }
            };
        }
        macro_rules! read_bytes_vec {
            ($size: expr) => {
                {
                    let mut d = vec![0u8; $size];
                    for i in 0..$size {
                        d[i] = data.read_u8().unwrap();
                    }
                    d
                }
            };
        }
        macro_rules! read_bool {
            () => {
                {
                    data.read_u32::<LittleEndian>().unwrap() == 1
                }
            };
        }
        
        // We'll start unpacking our data.win, the data should start
        // with a "00 00 00 00" u32, which indicates the start of
        // the data.win, as "FORM".

        // We'll start by defining all of the chunks.

        #[derive(Default, Debug)]
        struct FormChunk {
            size: u32
        }
        
        #[derive(Default, Debug)]
        struct Gen8Chunk {
            is_debugged_disabled: bool,
            bytecode_version: u8,
            unknown: u16,
            filename: String,
            config: String,
            last_obj: u32,
            last_tile: u32,
            game_id: u32,
            guid_data: [u8; 16],
            name: String,
            major: u32,
            minor: u32,
            release: u32,
            build: u32,
            default_window_width: u32,
            default_window_height: u32,
            info: u32,
            license_crc32: u32,
            license_md5: [u8; 16],
            timestamp: u64,
            display_name: String,
            active_targets: u64,
            function_classifications: u64,
            steam_app_id: u32,
            debugger_port: u32,
            room_order: Vec<u32>
        }
        
        #[derive(Default, Debug)]
        struct OptnChunk {
            unknown1: u32,
            unknown2: u32,
            info: u64,
            scale: i32,
            window_color: u32,
            color_depth: u32,
            resolution: u32,
            frequency: u32,
            vertex_sync: u32,
            priority: u32,
            back_image: u32,
            front_image: u32,
            load_image: u32,
            load_alpha: u32,
            constants: HashMap<String, String>
        }
        
        #[derive(Default, Debug)]
        struct LangChunk {
            unknown1: u32,
            language_count: u32,
            entry_count: u32
        }
        
        #[derive(Default, Debug)]
        struct ExtnChunk {
            data: Vec<ExtnData>,
            product_id_data: Vec<[u8; 16]>
        
        }#[derive(Default, Debug)]
        struct ExtnData {
            empty_string: String,
            extension_name: String,
            class_name: String,
            extension_includes: Vec<ExtnIncl>
        }
        #[derive(Default, Debug)]
        struct ExtnIncl {
            filename: String,
            end_function: String,
            start_function: String,
            file_kind: i32,
            file_functions: Vec<ExtnFunc>
        }
        #[derive(Default, Debug)]
        struct ExtnFunc {
            name: String,
            id: u32,
            function_kind: u32,
            return_kind: u32,
            external_name: String,
            arguments: Vec<u32>
        }
        
        #[derive(Default, Debug)]
        struct SondChunk {
            data: Vec<SondData>
        }
        #[derive(Default, Debug)]
        struct SondData {
            name: String,
            flags: u32,
            kind: String,
            file: String,
            effects: u32,
            volume: f32,
            pitch: f32,
            group_id: u32,
            audio_id: u32
        }
        
        #[derive(Default, Debug)]
        struct ArgpChunk {
            names: Vec<String>
        }
        
        #[derive(Default, Debug)]
        struct SprtChunk {
            data: Vec<SprtData>
        }
        #[derive(Default, Debug)]
        struct SprtData {
            name: String,
            width: u32,
            height: u32,
            margin_left: i32,
            margin_right: i32,
            margin_bottom: i32,
            margin_top: i32,
            transparent: bool,
            smooth: bool,
            preload: bool,
            bbox_mode: u32,
            sep_masks: u32,
            origin_x: i32,
            origin_y: i32,
            textures: Vec<u32>,
            mask_size: u32,
            mask_data: Vec<Vec<u8>>
        }
        
        #[derive(Default, Debug)]
        struct BgndChunk {
            data: Vec<BgndData>
        }
        #[derive(Default, Debug)]
        struct BgndData {
            name: String,
            transparent: bool,
            smooth: bool,
            preload: bool,
            texture: u32
        }
        
        #[derive(Default, Debug)]
        struct PathChunk {
            data: Vec<PathData>
        }
        #[derive(Default, Debug)]
        struct PathData {
            name: String,
            smooth: bool,
            closed: bool,
            precision: u32,
            points: Vec<PathPoint>
        }
        #[derive(Default, Debug)]
        struct PathPoint {
            x: f32,
            y: f32,
            speed: f32
        }

        #[derive(Default, Debug)]
        struct ScptChunk {
            data: Vec<ScptData>
        }
        #[derive(Default, Debug)]
        struct ScptData {
            name: String,
            id: u32
        }

        #[derive(Default, Debug)]
        struct GlobChunk {
            items: Vec<u32>
        }

        #[derive(Default, Debug)]
        struct ShdrChunk {
            data: Vec<ShdrData>
        }
        #[derive(Default, Debug)]
        struct ShdrData {
            name: String,
            kind: u32,
            glsl_es_vertex: String,
            glsl_es_fragment: String,
            glsl_vertex: String,
            glsl_fragment: String,
            hlsl9_vertex: String,
            hlsl9_fragment: String,
            hlsl11_vertex_data: u32,
            hlsl11_pixel_data: u32,
            vertex_shader_attributes: Vec<String>,
            version: u32,
            pssl_vertex_data: u32,
            pssl_pixel_data: u32,
            cg_psvita_vertex_data: u32,
            cg_psvita_pixel_data: u32,
            cg_ps3_vertex_data: u32,
            cg_ps3_pixel_data: u32,
            padding: [u8; 24] // No fucking idea what's here, it's just null data
        }

        #[derive(Default, Debug)]
        struct FontChunk {
            data: Vec<FontData>,
            buffer: Vec<u8>
        }
        #[derive(Default, Debug)]
        struct FontData {
            name: String,
            display_name: String,
            em_size: u32,
            bold: bool,
            italic: bool,
            range_start: u16,
            charset: u8,
            antialiasing: u8,
            range_end: u16,
            unknown1: u16,
            texture: u32,
            scale_x: f32,
            scale_y: f32,
            glyph: Vec<FontGlyph>
        }
        #[derive(Default, Debug)]
        #[allow(dead_code)]
        struct FontGlyph {
            character: u16,
            source_x: u16,
            source_y: u16,
            source_width: u16,
            source_height: u16,
            shift: i16,
            offset: i16,
            kerning: Vec<FontGlyphKerning>
        }
        #[derive(Default, Debug)]
        #[allow(dead_code)]
        struct FontGlyphKerning {
            character: i16,
            shift_modifier: i16
        }

        #[derive(Default, Debug)]
        struct ObjtChunk {
            data: Vec<ObjtData>
        }
        #[derive(Default, Debug)]
        struct ObjtData {
            name: String,
            sprite: i32,
            visible: bool,
            solid: bool,
            depth: i32,
            persistent: bool,
            parent: i32,
            texture_mask_id: i32,
            uses_physics: bool,
            is_sensor: bool,
            collision_shape: u32,
            density: f32,
            restitution: f32,
            group: u32,
            linear_dampling: f32,
            angular_dampling: f32,
            physics_shape_vertices: Vec<ObjtPhysicsVertex>,
            friction: f32,
            awake: bool,
            kinematic: bool,
            events: Vec<Vec<ObjtEvent>>
        }
        #[derive(Default, Debug)]
        struct ObjtPhysicsVertex {
            x: f32,
            y: f32
        }
        #[derive(Default, Debug)]
        struct ObjtEvent {
            event_subtype: u32,
            event_action: Vec<ObjtEventAction>
        }
        #[derive(Default, Debug)]
        struct ObjtEventAction {
            lib_id: u32,
            id: u32,
            kind: u32,
            use_relative: bool,
            is_question: bool,
            use_apply_to: bool,
            exe_type: u32,
            action_name: String,
            code_id: u32,
            argument_count: u32,
            who: i32,
            relative: bool,
            is_not: bool,
            unknown1: u32
        }

        #[derive(Default, Debug)]
        struct RoomChunk {
            data: Vec<RoomData>
        }
        #[derive(Default, Debug)]
        struct RoomData {
            name: String,
            caption: String,
            width: u32,
            height: u32,
            speed: u32,
            persistent: bool,
            background_color: u32,
            draw_background_color: bool,
            creation_code_id: u32,
            flags: u32,
            backgrounds: Vec<RoomBackground>,
            views: Vec<RoomView>,
            game_objects: Vec<RoomObject>,
            tiles: Vec<RoomTile>,
            world: bool,
            top: u32,
            left: u32,
            right: u32,
            bottom: u32,
            gravity_x: f32,
            gravity_y: f32,
            meters_per_pixel: f32
        }
        #[derive(Default, Debug)]
        struct RoomBackground {
            enabled: bool,
            foreground: bool,
            definition: i32,
            x: i32,
            y: i32,
            tile_x: i32,
            tile_y: i32,
            speed_x: i32,
            speed_y: i32,
            stretch: bool
        }
        #[derive(Default, Debug)]
        struct RoomView {
            enabled: bool,
            view_x: i32,
            view_y: i32,
            view_width: i32,
            view_height: i32,
            port_x: i32,
            port_y: i32,
            port_width: i32,
            port_height: i32,
            border_x: u32,
            border_y: u32,
            speed_x: i32,
            speed_y: i32,
            object_id: i32
        }
        #[derive(Default, Debug)]
        struct RoomObject {
            x: i32,
            y: i32,
            object_id: i32,
            instance_id: u32,
            creation_code: i32,
            scale_x: f32,
            scale_y: f32,
            color: u32,
            angle: f32,
            pre_creation_code: i32
        }
        #[derive(Default, Debug)]
        struct RoomTile {
            x: i32,
            y: i32,
            background_id: i32,
            source_x: u32,
            source_y: u32,
            width: u32,
            height: u32,
            tile_depth: i32,
            instance_id: u32,
            scale_x: f32,
            scale_y: f32,
            color: u32
        }

        #[derive(Default, Debug)]
        struct TpagChunk {
            data: Vec<TpagData>
        }
        #[derive(Default, Debug)]
        struct TpagData {
            source_x: u16,
            source_y: u16,
            source_width: u16,
            source_height: u16,
            target_x: u16,
            target_y: u16,
            target_width: u16,
            target_height: u16,
            bounding_width: u16,
            bounding_height: u16,
            texture_id: i16
        }

        let mut form = FormChunk { ..Default::default() };
        let mut gen8 = Gen8Chunk { ..Default::default() };
        let mut optn = OptnChunk { ..Default::default() };
        let mut lang = LangChunk { ..Default::default() };
        let mut extn = ExtnChunk { ..Default::default() };
        let mut sond = SondChunk { ..Default::default() };
        let mut argp = ArgpChunk { ..Default::default() };
        let mut sprt = SprtChunk { ..Default::default() };
        let mut bgnd = BgndChunk { ..Default::default() };
        let mut path = PathChunk { ..Default::default() };
        let mut scpt = ScptChunk { ..Default::default() };
        let mut glob = GlobChunk { ..Default::default() };
        let mut shdr = ShdrChunk { ..Default::default() };
        let mut font = FontChunk { ..Default::default() };
        // TMLN goes here, if I add it one day.
        let mut objt = ObjtChunk { ..Default::default() };
        let mut room = RoomChunk { ..Default::default() };

        // Unserializer

        {
            info!("Start unserializing...");
            // Start reading the data...

            /*let mut file = BufWriter::new(File::create("dump").unwrap());
            file.write_all(&data.clone().into_inner()).unwrap();
            file.flush().unwrap();
            drop(file);*/

            // FORM Chunk

            {
                data.seek(SeekFrom::Current(4)).unwrap(); // Ignore chunk name
                form.size = data.read_u32::<LittleEndian>().unwrap();
            }

            info!("Offset: {}", data.position());
            // GEN8 Chunk

            {
                data.seek(SeekFrom::Current(8)).unwrap(); // Ignore chunk name and size
                gen8.is_debugged_disabled = read_u8!() != 0;
                gen8.bytecode_version = read_u8!();
                if gen8.bytecode_version != 16 {
                    info!("Warning: Expected Bytecode 16, found Bytecode {}", gen8.bytecode_version);
                }
                gen8.unknown = read_u16!();
                gen8.filename = read_string!();
                gen8.config = read_string!();
                gen8.last_obj = read_u32!();
                gen8.last_tile = read_u32!();
                gen8.game_id = read_u32!();
                gen8.guid_data = read_bytes!(16);
                gen8.name = read_string!();
                gen8.major = read_u32!();
                gen8.minor = read_u32!();
                gen8.release = read_u32!();
                gen8.build = read_u32!();
                gen8.default_window_width = read_u32!();
                gen8.default_window_height = read_u32!();
                gen8.info = read_u32!();
                gen8.license_crc32 = read_u32!();
                gen8.license_md5 = read_bytes!(16);
                gen8.timestamp = read_u64!();
                gen8.display_name = read_string!();
                gen8.active_targets = read_u64!();
                gen8.function_classifications = read_u64!();
                gen8.steam_app_id = read_u32!();
                gen8.debugger_port = read_u32!();
                gen8.room_order.resize(read_u32!() as usize, 0);
                for i in 0..gen8.room_order.len() {
                    gen8.room_order[i] = read_u32!();
                }

                //info!("{:?}", gen8);
                info!("GEN8 OK!");
            }

            info!("Offset: {}", data.position());
            // OPTN Chunk

            {
                data.seek(SeekFrom::Current(8)).unwrap(); // Ignore chunk name and size
                optn.unknown1 = read_u32!();
                optn.unknown2 = read_u32!();
                optn.info = read_u64!();
                optn.scale = read_i32!();
                optn.window_color = read_u32!();
                optn.color_depth = read_u32!();
                optn.resolution = read_u32!();
                optn.frequency = read_u32!();
                optn.vertex_sync = read_u32!();
                optn.priority = read_u32!();
                optn.back_image = read_u32!();
                optn.front_image = read_u32!();
                optn.load_image = read_u32!();
                optn.load_alpha = read_u32!();
                for _ in 0..read_u32!() {
                    let name = read_string!();
                    let value = read_string!();
                    optn.constants.insert(name, value);
                }

                //info!("{:?}", optn);
                info!("OPTN OK!");
            }

            info!("Offset: {}", data.position());
            // LANG Chunk

            {
                data.seek(SeekFrom::Current(8)).unwrap(); // Ignore chunk name and size
                lang.unknown1 = read_u32!();
                lang.language_count = read_u32!();
                lang.entry_count = read_u32!(); // Very vague implementation... should work as it's unused.

                //info!("{:?}", lang);
                info!("LANG OK!");
            }

            info!("Offset: {}", data.position());
            // EXTN Chunk

            {
                data.seek(SeekFrom::Current(8)).unwrap(); // Ignore chunk name and size
                let mut extn_ptr = Vec::new();
                for _ in 0..read_u32!() {
                    extn_ptr.push(read_u32!());
                }
                for ptr in &extn_ptr {
                    data.seek(SeekFrom::Start(*ptr as u64)).unwrap();
                    let mut entry = ExtnData {
                        ..Default::default()
                    };

                    entry.empty_string = read_string!();
                    entry.extension_name = read_string!();
                    entry.class_name = read_string!();

                    let mut file_ptrs = Vec::new();
                    for _ in 0..read_u32!() {
                        file_ptrs.push(read_u32!());
                    }

                    for file_ptr in file_ptrs {
                        data.seek(SeekFrom::Start(file_ptr as u64)).unwrap();
                        let mut file_entry = ExtnIncl {
                            ..Default::default()
                        };

                        file_entry.filename = read_string!();
                        file_entry.end_function = read_string!();
                        file_entry.start_function = read_string!();
                        file_entry.file_kind = read_i32!();

                        let mut func_ptrs = Vec::new();
                        for _ in 0..read_u32!() {
                            func_ptrs.push(read_u32!());
                        }

                        for func_ptr in func_ptrs {
                            data.seek(SeekFrom::Start(func_ptr as u64)).unwrap();
                            let mut func_entry = ExtnFunc {
                                ..Default::default()
                            };

                            func_entry.name = read_string!();
                            func_entry.id = read_u32!();
                            func_entry.function_kind = read_u32!();
                            func_entry.return_kind = read_u32!();
                            func_entry.external_name = read_string!();
                            for _ in 0..read_u32!() {
                                func_entry.arguments.push(read_u32!());
                            }

                            file_entry.file_functions.push(func_entry);
                        }

                        entry.extension_includes.push(file_entry);
                    }

                    extn.data.push(entry);
                }

                for _ in 0..extn_ptr.len() {
                    extn.product_id_data.push(read_bytes!(16));
                }

                //info!("{:?}", extn);
                info!("EXTN OK!");
            }
            
            info!("Offset: {}", data.position());
            // SOND Chunk

            {
                data.seek(SeekFrom::Current(8)).unwrap(); // Ignore chunk name and size
                let mut sond_ptr = Vec::new();
                for _ in 0..read_u32!() {
                    sond_ptr.push(read_u32!());
                }
                for ptr in sond_ptr {
                    data.seek(SeekFrom::Start(ptr as u64)).unwrap();
                    let mut entry = SondData {
                        ..Default::default()
                    };

                    entry.name = read_string!();
                    entry.flags = read_u32!();
                    entry.kind = read_string!();
                    entry.file = read_string!();
                    entry.effects = read_u32!();
                    entry.volume = read_f32!();
                    entry.pitch = read_f32!();
                    entry.group_id = read_u32!();
                    entry.audio_id = read_u32!();

                    sond.data.push(entry);
                }

                //info!("{:?}", sond);
                info!("SOND OK!");
            }

            info!("Offset: {}", data.position());
            // ARGP Chunk

            {
                data.seek(SeekFrom::Current(8)).unwrap(); // Ignore chunk name and size
                let mut argp_ptr = Vec::new();
                for _ in 0..read_u32!() {
                    argp_ptr.push(read_u32!());
                }

                for ptr in argp_ptr {
                    data.seek(SeekFrom::Start(ptr as u64)).unwrap();
                    argp.names.push(read_string!());
                }

                //info!("{:?}", argp);
                info!("ARGP OK!");
            }

            info!("Offset: {}", data.position());
            // SPRT Chunk

            {
                data.seek(SeekFrom::Current(8)).unwrap(); // Ignore chunk name and size
                let mut sprt_ptr = Vec::new();
                for _ in 0..read_u32!() {
                    sprt_ptr.push(read_u32!());
                }

                for (index, ptr) in sprt_ptr.iter().enumerate() {
                    data.seek(SeekFrom::Start(*ptr as u64)).unwrap();
                    let mut entry = SprtData {
                        ..Default::default()
                    };

                    entry.name = read_string!();
                    entry.height = read_u32!();
                    entry.origin_x = read_i32!();
                    entry.margin_left = read_i32!();
                    entry.margin_right = read_i32!();
                    entry.margin_bottom = read_i32!();
                    entry.width = read_u32!();
                    entry.transparent = read_bool!();
                    entry.smooth = read_bool!();
                    entry.preload = read_bool!();
                    entry.bbox_mode = read_u32!();
                    entry.sep_masks = read_u32!();
                    entry.margin_top = read_i32!();
                    entry.origin_y = read_i32!();
                    for _ in 0..read_u32!() {
                        entry.textures.push(read_u32!());
                    }
                    
                    entry.mask_size = read_u32!();

                    for _ in 0..entry.mask_size {
                        entry.mask_data.push(read_bytes_vec!(((entry.width + 7) / 8 * entry.height) as usize));
                    }
                    if index + 1 < sprt_ptr.len() {
                        let nptr = sprt_ptr[index + 1];
                        let size = (data.position() as i64 - nptr as i64).unsigned_abs() as usize;
                        if size > 0 {
                            entry.mask_data.push(read_bytes_vec!(size));
                        }
                    }
                    sprt.data.push(entry);
                }

                //info!("{:?}", sprt);
                info!("SPRT OK!");
            }

            info!("Offset: {}", data.position());
            // BGND Chunk
            
            {
                data.seek(SeekFrom::Current(8)).unwrap(); // Ignore chunk name and size
                let mut bgnd_ptr = Vec::new();
                for _ in 0..read_u32!() {
                    bgnd_ptr.push(read_u32!());
                }

                for ptr in bgnd_ptr {
                    data.seek(SeekFrom::Start(ptr as u64)).unwrap();
                    let mut entry = BgndData {
                        ..Default::default()
                    };

                    entry.name = read_string!();
                    entry.transparent = read_bool!();
                    entry.smooth = read_bool!();
                    entry.preload = read_bool!();
                    entry.texture = read_u32!();

                    bgnd.data.push(entry);
                }

                //info!("{:?}", bgnd);
                info!("BGND OK!");
            }

            info!("Offset: {}", data.position());
            // PATH Chunk

            {
                data.seek(SeekFrom::Current(8)).unwrap(); // Ignore chunk name and size
                let mut path_ptr = Vec::new();
                for _ in 0..read_u32!() {
                    path_ptr.push(read_u32!());
                }

                for ptr in path_ptr {
                    data.seek(SeekFrom::Start(ptr as u64)).unwrap();
                    let mut entry = PathData {
                        ..Default::default()
                    };

                    entry.name = read_string!();
                    entry.smooth = read_bool!();
                    entry.closed = read_bool!();
                    entry.precision = read_u32!();
                    for _ in 0..read_u32!() {
                        let mut point = PathPoint {
                            ..Default::default()
                        };

                        point.x = read_f32!();
                        point.y = read_f32!();
                        point.speed = read_f32!();

                        entry.points.push(point);
                    }

                    path.data.push(entry);
                }

                //info!("{:?}", path);
                info!("PATH OK!");
            }

            info!("Offset: {}", data.position());
            // SCPT Chunk

            {
                data.seek(SeekFrom::Current(8)).unwrap(); // Ignore chunk name and size
                let mut scpt_ptr = Vec::new();
                for _ in 0..read_u32!() {
                    scpt_ptr.push(read_u32!());
                }

                for ptr in scpt_ptr {
                    data.seek(SeekFrom::Start(ptr as u64)).unwrap();
                    let mut entry = ScptData {
                        ..Default::default()
                    };

                    entry.name = read_string!();
                    entry.id = read_u32!();

                    scpt.data.push(entry);
                }
            }

            info!("Offset: {}", data.position());
            // GLOB Chunk

            {
                data.seek(SeekFrom::Current(8)).unwrap(); // Ignore chunk name and size
                for _ in 0..read_u32!() {
                    glob.items.push(read_u32!());
                }

                //info!("{:?}", glob);
                info!("GLOB OK!");
            }

            info!("Offset: {}", data.position());
            // SHDR Chunk

            {
                data.seek(SeekFrom::Current(8)).unwrap(); // Ignore chunk name and size
                let mut shdr_ptr = Vec::new();
                for _ in 0..read_u32!() {
                    shdr_ptr.push(read_u32!());
                }

                for ptr in shdr_ptr {
                    data.seek(SeekFrom::Start(ptr as u64)).unwrap();
                    let mut entry = ShdrData {
                        ..Default::default()
                    };

                    entry.name = read_string!();
                    entry.kind = read_u32!();
                    entry.glsl_es_vertex = read_string!();
                    entry.glsl_es_fragment = read_string!();
                    entry.glsl_vertex = read_string!();
                    entry.glsl_fragment = read_string!();
                    entry.hlsl9_vertex = read_string!();
                    entry.hlsl9_fragment = read_string!();
                    entry.hlsl11_vertex_data = read_u32!();
                    entry.hlsl11_pixel_data = read_u32!();
                    for _ in 0..read_u32!() {
                        entry.vertex_shader_attributes.push(read_string!());
                    }
                    entry.version = read_u32!();
                    entry.pssl_vertex_data = read_u32!();
                    entry.pssl_pixel_data = read_u32!();
                    entry.cg_psvita_vertex_data = read_u32!();
                    entry.cg_psvita_pixel_data = read_u32!();
                    entry.cg_ps3_vertex_data = read_u32!();
                    entry.cg_ps3_pixel_data = read_u32!();
                    entry.padding = read_bytes!(24);

                    shdr.data.push(entry);
                }

                //info!("{:?}", shdr);
                info!("SHDR OK!");
            }

            info!("Offset: {}", data.position());
            // FONT Chunk

            {
                data.seek(SeekFrom::Current(8)).unwrap(); // Ignore chunk name and size
                let mut font_ptr = Vec::new();
                for _ in 0..read_u32!() {
                    font_ptr.push(read_u32!());
                }

                for ptr in font_ptr {
                    data.seek(SeekFrom::Start(ptr as u64)).unwrap();
                    let mut entry = FontData {
                        ..Default::default()
                    };

                    entry.name = read_string!();
                    entry.display_name = read_string!();
                    entry.em_size = read_u32!();
                    entry.bold = read_bool!();
                    entry.italic = read_bool!();
                    entry.range_start = read_u16!();
                    entry.charset = read_u8!();
                    entry.antialiasing = read_u8!();
                    entry.range_end = read_u16!();
                    entry.unknown1 = read_u16!();
                    entry.texture = read_u32!();
                    entry.scale_x = read_f32!();
                    entry.scale_y = read_f32!();
                    
                    let mut glyph_ptrs = Vec::new();
                    for _ in 0..read_u32!() {
                        glyph_ptrs.push(read_u32!());
                    }

                    for glyph_ptr in glyph_ptrs {
                        data.seek(SeekFrom::Start(glyph_ptr as u64)).unwrap();
                        let mut glyph = FontGlyph {
                            ..Default::default()
                        };

                        glyph.character = read_u16!();
                        glyph.source_x = read_u16!();
                        glyph.source_y = read_u16!();
                        glyph.source_width = read_u16!();
                        glyph.source_height = read_u16!();
                        glyph.shift = read_i16!();
                        glyph.offset = read_i16!();

                        if read_u16!() != 0 {
                            warn!("Glyph has Kerning!!! Offset: {}", data.position());
                        }

                        entry.glyph.push(glyph);
                    }

                    font.data.push(entry);
                }

                font.buffer = read_bytes_vec!(512);

                //info!("{:?}", font);
                info!("FONT OK!");
            }

            info!("Offset: {}", data.position());
            // TMLN Chunk

            {
                data.seek(SeekFrom::Current(8)).unwrap(); // Ignore chunk name and size
                let timeline_amount = read_u32!();

                if timeline_amount > 0 {
                    error!("There's {} timelines, while expecting 0 timelines.", timeline_amount);
                    return Ok(());
                }
            }

            info!("Offset: {}", data.position());
            // OBJT Chunk

            {
                data.seek(SeekFrom::Current(8)).unwrap(); // Ignore chunk name and size
                let mut objt_ptr = Vec::new();
                for _ in 0..read_u32!() {
                    objt_ptr.push(read_u32!());
                }
                for ptr in objt_ptr {
                    data.seek(SeekFrom::Start(ptr as u64)).unwrap();
                    let mut entry = ObjtData {
                        ..Default::default()
                    };

                    entry.name = read_string!();
                    entry.sprite = read_i32!();
                    entry.visible = read_bool!();
                    entry.solid = read_bool!();
                    entry.depth = read_i32!();
                    entry.persistent = read_bool!();
                    entry.parent = read_i32!();
                    entry.texture_mask_id = read_i32!();
                    entry.uses_physics = read_bool!();
                    entry.is_sensor = read_bool!();
                    entry.collision_shape = read_u32!();
                    entry.density = read_f32!();
                    entry.restitution = read_f32!();
                    entry.group = read_u32!();
                    entry.linear_dampling = read_f32!();
                    entry.angular_dampling = read_f32!();
                    let physics_shape_vertex_count = read_u32!();
                    entry.friction = read_f32!();
                    entry.awake = read_bool!();
                    entry.kinematic = read_bool!();
                    for _ in 0..physics_shape_vertex_count {
                        let mut vertex = ObjtPhysicsVertex {
                            ..Default::default()
                        };
                        vertex.x = read_f32!();
                        vertex.y = read_f32!();

                        entry.physics_shape_vertices.push(vertex);
                    }
                    let mut event_ptrs = Vec::new();
                    for _ in 0..read_u32!() {
                        event_ptrs.push(read_u32!());
                    }

                    for event_ptr in event_ptrs {
                        data.seek(SeekFrom::Start(event_ptr as u64)).unwrap();
                        let mut subevent_ptrs = Vec::new();
                        for _ in 0..read_u32!() {
                            subevent_ptrs.push(read_u32!());
                        }

                        let mut events = Vec::new();

                        for subevent_ptr in subevent_ptrs {
                            data.seek(SeekFrom::Start(subevent_ptr as u64)).unwrap();
                            let mut event = ObjtEvent {
                                ..Default::default()
                            };

                            event.event_subtype = read_u32!();
                            let mut action_ptrs = Vec::new();
                            for _ in 0..read_u32!() {
                                action_ptrs.push(read_u32!());
                            }

                            for action_ptr in action_ptrs {
                                data.seek(SeekFrom::Start(action_ptr as u64)).unwrap();
                                let mut action = ObjtEventAction {
                                    ..Default::default()
                                };

                                action.lib_id = read_u32!();
                                action.id = read_u32!();
                                action.kind = read_u32!();
                                action.use_relative = read_bool!();
                                action.is_question = read_bool!();
                                action.use_apply_to = read_bool!();
                                action.exe_type = read_u32!();
                                action.action_name = read_string!();
                                action.code_id = read_u32!();
                                action.argument_count = read_u32!();
                                action.who = read_i32!();
                                action.relative = read_bool!();
                                action.is_not = read_bool!();
                                action.unknown1 = read_u32!();

                                event.event_action.push(action);
                            }

                            events.push(event);
                        }

                        entry.events.push(events);
                    }

                    objt.data.push(entry);
                }

                //info!("{:?}", objt);
                info!("OBJT OK!");
            }

            info!("Offset: {}", data.position());
            // ROOM Chunk

            {
                data.seek(SeekFrom::Current(8)).unwrap(); // Ignore chunk name and size
                let mut room_ptr = Vec::new();
                for _ in 0..read_u32!() {
                    room_ptr.push(read_u32!());
                }
                for ptr in room_ptr {
                    data.seek(SeekFrom::Start(ptr as u64)).unwrap();
                    let mut entry = RoomData {
                        ..Default::default()
                    };

                    entry.name = read_string!();
                    entry.caption = read_string!();
                    entry.width = read_u32!();
                    entry.height = read_u32!();
                    entry.speed = read_u32!();
                    entry.persistent = read_bool!();
                    entry.background_color = read_u32!();
                    entry.draw_background_color = read_bool!();
                    entry.creation_code_id = read_u32!();
                    entry.flags = read_u32!();
                    let background_ptr = read_u32!();
                    let view_ptr = read_u32!();
                    let objects_ptr = read_u32!();
                    let tiles_ptr = read_u32!();
                    entry.world = read_bool!();
                    entry.top = read_u32!();
                    entry.left = read_u32!();
                    entry.right = read_u32!();
                    entry.bottom = read_u32!();
                    entry.gravity_x = read_f32!();
                    entry.gravity_y = read_f32!();
                    entry.meters_per_pixel = read_f32!();

                    data.seek(SeekFrom::Start(background_ptr as u64)).unwrap();
                    let mut background_ptrs = Vec::new();
                    for _ in 0..read_u32!() {
                        background_ptrs.push(read_u32!());
                    }
                    for background_ptr in background_ptrs {
                        data.seek(SeekFrom::Start(background_ptr as u64)).unwrap();
                        let mut background = RoomBackground {
                            ..Default::default()
                        };

                        background.enabled = read_bool!();
                        background.foreground = read_bool!();
                        background.definition = read_i32!();
                        background.x = read_i32!();
                        background.y = read_i32!();
                        background.tile_x = read_i32!();
                        background.tile_y = read_i32!();
                        background.speed_x = read_i32!();
                        background.speed_y = read_i32!();
                        background.stretch = read_bool!();

                        entry.backgrounds.push(background);
                    }
                    data.seek(SeekFrom::Start(view_ptr as u64)).unwrap();
                    let mut view_ptrs = Vec::new();
                    for _ in 0..read_u32!() {
                        view_ptrs.push(read_u32!());
                    }
                    for view_ptr in view_ptrs {
                        data.seek(SeekFrom::Start(view_ptr as u64)).unwrap();
                        let mut view = RoomView {
                            ..Default::default()
                        };

                        view.enabled = read_bool!();
                        view.view_x = read_i32!();
                        view.view_y = read_i32!();
                        view.view_width = read_i32!();
                        view.view_height = read_i32!();
                        view.port_x = read_i32!();
                        view.port_y = read_i32!();
                        view.port_width = read_i32!();
                        view.port_height = read_i32!();
                        view.border_x = read_u32!();
                        view.border_y = read_u32!();
                        view.speed_x = read_i32!();
                        view.speed_y = read_i32!();
                        view.object_id = read_i32!();

                        entry.views.push(view);
                    }
                    data.seek(SeekFrom::Start(objects_ptr as u64)).unwrap();
                    let mut objects_ptrs = Vec::new();
                    for _ in 0..read_u32!() {
                        objects_ptrs.push(read_u32!());
                    }
                    for objects_ptr in objects_ptrs {
                        data.seek(SeekFrom::Start(objects_ptr as u64)).unwrap();
                        let mut object = RoomObject {
                            ..Default::default()
                        };

                        object.x = read_i32!();
                        object.y = read_i32!();
                        object.object_id = read_i32!();
                        object.instance_id = read_u32!();
                        object.creation_code = read_i32!();
                        object.scale_x = read_f32!();
                        object.scale_y = read_f32!();
                        object.color = read_u32!();
                        object.angle = read_f32!();
                        object.pre_creation_code = read_i32!();

                        entry.game_objects.push(object);
                    }
                    data.seek(SeekFrom::Start(tiles_ptr as u64)).unwrap();
                    let mut tiles_ptrs = Vec::new();
                    for _ in 0..read_u32!() {
                        tiles_ptrs.push(read_u32!());
                    }
                    for tiles_ptr in tiles_ptrs {
                        data.seek(SeekFrom::Start(tiles_ptr as u64)).unwrap();
                        let mut tile = RoomTile {
                            ..Default::default()
                        };

                        tile.x = read_i32!();
                        tile.y = read_i32!();
                        tile.background_id = read_i32!();
                        tile.source_x = read_u32!();
                        tile.source_y = read_u32!();
                        tile.width = read_u32!();
                        tile.height = read_u32!();
                        tile.tile_depth = read_i32!();
                        tile.instance_id = read_u32!();
                        tile.scale_x = read_f32!();
                        tile.scale_y = read_f32!();
                        tile.color = read_u32!();

                        entry.tiles.push(tile);
                    }

                    room.data.push(entry);
                }

                //info!("{:?}", room);
                info!("ROOM OK!");
            }

            info!("Offset: {}", data.position());
            // DAFL Chunk

            {
                data.seek(SeekFrom::Current(8)).unwrap(); // Ignore chunk name and size
                info!("DAFL OK!");
            }

            info!("Offset: {}", data.position());
            // TPAG Chunk

            {
                data.seek(SeekFrom::Current(8)).unwrap(); // Ignore chunk name and size
                let mut tpag_ptr = Vec::new();
                for _ in 0..read_u32!() {
                    tpag_ptr.push(read_u32!());
                }
                for ptr in tpag_ptr {
                    data.seek(SeekFrom::Start(ptr as u64)).unwrap();
                    let mut entry = RoomData {
                        ..Default::default()
                    };
                }

                //info!("{:?}", tpag);
                info!("TPAG OK!");
            }

            info!("Offset: {}", data.position());
            //
        }

        // Temmie Flakes serializer
        
        {
            info!("Start serializing...");
            let mut data: Vec<u8> = Vec::new();

            let mut chunk_offsets: Vec<usize> = Vec::new();
            let mut string_pointers: HashMap<String, (Option<u32>, Vec<usize>)> = HashMap::new(); // HashMap<String text, (String pointer, Vec<text pointing>)>
            macro_rules! write_chunk {
                ($name: expr) => {
                    {
                        chunk_offsets.push(data.len());
                        data.extend($name.as_bytes());
                        data.extend([0x00, 0x00, 0x00, 0x00]); // Size
                    }
                };
            }
            macro_rules! cache_string {
                ($text: expr) => {
                    {
                        let text = $text.to_string();
                        if string_pointers.contains_key(&text) {
                            string_pointers.get_mut(&text).unwrap()
                                .0 = Some((data.len() + 4) as u32);
                        }else{
                            string_pointers.insert(text, (
                                Some((data.len() + 4) as u32),
                                Vec::new()
                            ));
                        }
                    }
                };
            }
            macro_rules! write_string {
                ($text: expr) => {
                    let text = $text.to_string();
                    cache_string!(text);
                    data.extend((text.len() as u32).to_le_bytes());
                    data.extend(text.as_bytes());
                    data.push(0);
                };
                ($text: expr, $cache: expr) => {
                    let text = $text.to_string();
                    if $cache {
                        cache_string!(text);
                    }
                    data.extend((text.len() as u32).to_le_bytes());
                    data.extend(text.as_bytes());
                    data.push(0);
                };
            }
            macro_rules! point_string {
                ($text: expr) => {
                    {
                        let text = $text.to_string();
                        if string_pointers.contains_key(&text) {
                            let d = string_pointers.get_mut(&text).unwrap();
                            if let Some(ptr) = d.0 {
                                write_value!(u32, ptr);
                            }else{
                                write_value!(u32, 0); // null
                                d.1.push(data.len());
                            }
                        }else{
                            string_pointers.insert(text, (
                                None,
                                vec![data.len()]
                            ));
                        }
                    }
                };
            }
            macro_rules! write_value {
                ($kind: ty, $value: expr) => {
                    data.extend(($value as $kind).to_le_bytes());
                };
            }
            macro_rules! poke_value {
                ($kind: ty, $offset: expr, $value: expr) => {
                    {
                        ($value as $kind).to_le_bytes().iter().enumerate()
                            .for_each(|(index, value)| {
                                data[$offset + index] = *value;
                            });
                    }
                };
            }

            // Sampling (tests for now)

            write_chunk!("FORM");
            let offset = data.len();
            write_value!(u32, 999);
            poke_value!(u32, offset, 128);
            point_string!("z"); // This will give a warning at runtime (when finalizing the serializing).
            write_string!("a");
            point_string!("a"); // Pointer here
            point_string!("abc"); // And also here
            write_string!("abc");

            // Finalize serializing

            {
                info!("Preparing to finalize serializing...");

                string_pointers.iter()
                    .for_each(|(text, (pointer, values))| {
                        if let Some(pointer) = *pointer {
                            values.iter()
                                .for_each(|value| {
                                    poke_value!(u32, *value, pointer);
                                });
                        }else{
                            warn!("{:?} was never given a pointer, while it was being used on offsets: {:?}.", text, values);
                        }
                    });
                
                info!("Finalized serializing.");
            }
            
            // Save data.win

            {
                info!("Saving data...");

                let mut f = BufWriter::new(File::create("data.win").unwrap());
                f.write_all(&data).unwrap();
                f.flush().unwrap();
                drop(f);

                let mut size = data.len() as f64;
                let mut kind = "byte(s)";

                while size >= 1000.0 {
                    size /= 1024.0;
                    kind = match kind {
                        "byte(s)" => "KB",
                        "KB" => "MB",
                        "MB" => "GB",
                        _ => ">GB"
                    }
                }
                
                info!("Saved data.win with size of {} {}",
                    if size.floor() != size {
                        format!("{:.2}", size)
                    }else{
                        format!("{}", size.floor())
                    }, kind);
            }
        }
    }

    Ok(())
}