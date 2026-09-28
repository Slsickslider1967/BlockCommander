

pub fn Add_Extern_Server(Exter_Dir: String)
{
    println!("adding a external server to server list");

    let dir = match get_dir() {
        Some(d) => d,
        None => return,
    };

    if !dir.exists()
    {
        println!("Directory does not exist");
        return;
    }


}