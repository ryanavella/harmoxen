use super::Delegate;
use crate::commands as cmds;
use crate::state::{self, State};
use crate::ui;
use crate::widget;
use druid::{commands as sys_cmds, Command, DelegateCtx, FileDialogOptions, FileSpec, Selector, Target};
use std::{fs, rc::Rc};

pub const IMPL_PROJECT_NEW: Selector = Selector::new("delegate.project-new");
pub const IMPL_PROJECT_OPEN: Selector = Selector::new("delegate.project-open");

impl Delegate {
	pub fn handle_fileops(
		&mut self,
		ctx: &mut DelegateCtx,
		cmd: &Command,
		data: &mut State,
		project_changed: &mut bool,
	) -> bool {
		let main_window = *data.main_window.clone().unwrap();
		match cmd {
			_ if cmd.is(cmds::PROJECT_NEW) => {
				if data.up_to_date {
					ctx.submit_command(IMPL_PROJECT_NEW, None);
				} else {
					ctx.submit_command(
						Command::new(widget::overlay::SHOW_MIDDLE, ui::modal::save::build(IMPL_PROJECT_NEW)),
						main_window.clone(),
					);
					self.after_save = Some(Box::new(|ctx: &mut DelegateCtx| {
						ctx.submit_command(IMPL_PROJECT_NEW, None);
					}));
				}
				false
			}
			_ if cmd.is(cmds::PROJECT_OPEN) => {
				if data.up_to_date {
					ctx.submit_command(IMPL_PROJECT_OPEN, None)
				} else {
					ctx.submit_command(
						Command::new(widget::overlay::SHOW_MIDDLE, ui::modal::save::build(IMPL_PROJECT_OPEN)),
						main_window.clone(),
					);
					self.after_save = Some(Box::new(|ctx: &mut DelegateCtx| {
						ctx.submit_command(IMPL_PROJECT_OPEN, None);
					}));
				}
				false
			}
			_ if cmd.is(cmds::PROJECT_SAVE_AS) => {
				ctx.submit_command(
					Command::new(
						sys_cmds::SHOW_SAVE_PANEL,
						FileDialogOptions::new().allowed_types(vec![FileSpec::new("Harmoxen project", &["hxp"])]),
					),
					Target::Window(*data.main_window.clone().unwrap()),
				);
				false
			}
			_ if cmd.is(cmds::PROJECT_SAVE) => {
				if let Some(path) = data.save_path.clone() {
					let project = state::Project::from_editors(&data.editors);
					let data = ron::to_string(&project).unwrap();
					fs::write(&*path, data).ok();
					if let Some(after_save) = self.after_save.take() {
						after_save(ctx);
					}
				} else {
					let xrp = FileSpec::new("Harmoxen project", &["hxp"]);
					ctx.submit_command(
						Command::new(
							sys_cmds::SHOW_SAVE_PANEL,
							FileDialogOptions::new().allowed_types(vec![xrp]).default_type(xrp),
						),
						Target::Window(*data.main_window.clone().unwrap()),
					);
				}
				false
			}
			_ if cmd.is(IMPL_PROJECT_NEW) => {
				let mut state = State::new();
				state.main_window = data.main_window.clone();
				*data = state;
				*project_changed = true;
				self.after_save = None;
				false
			}
			_ if cmd.is(IMPL_PROJECT_OPEN) => {
				ctx.submit_command(
					Command::new(
						sys_cmds::SHOW_OPEN_PANEL,
						FileDialogOptions::new().allowed_types(vec![FileSpec::new("Harmoxen project", &["hxp"])]),
					),
					*data.main_window.clone().unwrap(),
				);
				self.after_save = None;
				false
			}
			_ if cmd.is(sys_cmds::SAVE_FILE) => {
				if let Some(file_info) = cmd.get_unchecked(sys_cmds::SAVE_FILE) {
					data.up_to_date = true;
					data.save_path = Some(Rc::new(file_info.path().into()));
					let project = state::Project::from_editors(&data.editors);
					let project_str = ron::to_string(&project).unwrap();
					fs::write(file_info.path(), project_str).ok();

					if let Some(after_save) = self.after_save.take() {
						after_save(ctx);
					}
				}
				true
			}
			_ if cmd.is(sys_cmds::OPEN_FILE) => {
				let file_info = cmd.get_unchecked(sys_cmds::OPEN_FILE);
				if let Ok(project_string) = fs::read_to_string(file_info.path()) {
					if let Ok(project) = ron::from_str::<state::Project>(&project_string) {
						project.open(&mut data.editors);
						data.up_to_date = true;
						data.save_path = Some(Rc::new(file_info.path().into()));
						*project_changed = true;
					}
				}
				true
			}
			_ => true,
		}
	}
}
