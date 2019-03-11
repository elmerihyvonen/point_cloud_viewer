// Copyright 2016 Google Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use octree_web_viewer::backend_error::PointsViewerError;
use octree_web_viewer::state::AppState;
use octree_web_viewer::utils::start_octree_server;
//use point_viewer::octree;
use std::path::PathBuf;
//use std::sync::Arc;
use structopt::StructOpt;

/// HTTP web viewer for 3d points stored in OnDiskOctrees
#[derive(StructOpt, Debug)]
#[structopt(name = "points_web_viewer", about = "Visualizing points")]
pub struct CommandLineArguments {
    /// The octree directory to serve, including a trailing slash.
    #[structopt(name = "DIR")]
    octree_path: Option<String>,
    /// Port to listen on.
    #[structopt(default_value = "5433", long = "--port")]
    port: u16,
    /// IP string.
    #[structopt(default_value = "127.0.0.1", long = "--ip")]
    ip: String,
    /// Optional: prefix for path
    #[structopt(long = "--prefix")]
    path_prefix: Option<String>,
    /// Optional: suffix for path
    #[structopt(default_value = "/", long = "--suffix")]
    path_suffix: String,
    /// Optional: octree id
    #[structopt(long = "--uuid")]
    octree_id: Option<String>,
    /// Cache items
    #[structopt(default_value = "1", long = "--cache_items")]
    cache_max: usize,
}

pub fn state_from(args: CommandLineArguments) -> Result<(AppState, String), PointsViewerError> {
    //resolve suffix: trailing backslash
    let mut suffix = args.path_suffix;
    if !suffix.ends_with("/") {
        suffix.push('/');
    }

    let mut _uuid: String = "".to_string();
    let app_state = match args.octree_path {
        Some(path) => {
            let octree_directory = PathBuf::from(path);
            let mut prefix = octree_directory.parent().unwrap();
            if octree_directory.ends_with(&suffix) {
                prefix = prefix.parent().unwrap();
            }
            _uuid = octree_directory
                .strip_prefix(&prefix)?
                .to_str()
                .unwrap()
                .to_string();
            AppState::new(args.cache_max, prefix.to_str().unwrap(), suffix)
        }
        None => {
            let prefix = args
                .path_prefix
                .ok_or_else(|| {
                    PointsViewerError::NotFound(
                        "Input argoment Syntax is incorrect: check prefix".to_string(),
                    )
                })
                .unwrap();
            _uuid = args
                .octree_id
                .ok_or_else(|| {
                    PointsViewerError::NotFound(
                        "Input argoment Syntax is incorrect: check uuid".to_string(),
                    )
                })
                .unwrap();
            //path = PathBuf::from(Path::new(&prefix).join(&uuid).join(suffix));
            AppState::new(args.cache_max, prefix, suffix)
        }
    };

    Ok((app_state, _uuid))
}

fn main() {
    let args = CommandLineArguments::from_args();

    let ip_port = format!("{}:{}", args.ip, args.port);

    // initialize app state
    let (app_state, uuid) = state_from(args).unwrap();
    // The actix-web framework handles requests asynchronously using actors. If we need multi-threaded
    // write access to the Octree, instead of using an RwLock we should use the actor system.
    //put octree arc in cache
    let _ = app_state.insert_octree(uuid.clone());

    let sys = actix::System::new("octree-server");

    //let _ = start_octree_server(app_state, &ip_port, uuid);
    let _ = start_octree_server(app_state, &ip_port, uuid);

    println!("Starting http server: {}", &ip_port);
    let _ = sys.run();
}