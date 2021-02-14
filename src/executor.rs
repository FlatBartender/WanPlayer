// Copyright 2021 Flat Bartender <flat.bartender@gmail.com>
//
//    Licensed under the Apache License, Version 2.0 (the "License");
//    you may not use this file except in compliance with the License.
//    You may obtain a copy of the License at
//
//        http://www.apache.org/licenses/LICENSE-2.0
//
//    Unless required by applicable law or agreed to in writing, software
//    distributed under the License is distributed on an "AS IS" BASIS,
//    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//    See the License for the specific language governing permissions and
//    limitations under the License.

pub struct TokioExecutor;

impl iced::Executor for TokioExecutor {
    fn new() -> Result<Self, std::io::Error> {
        Ok(TokioExecutor {})
    }

    fn spawn(&self, future: impl Send + std::future::Future<Output = ()> + 'static) {
        tokio::spawn(future);
    }
}
