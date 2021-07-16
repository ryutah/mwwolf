use graphql_api_client::*;
use graphql_client::GraphQLQuery;
use yew::prelude::*;
use yewtil::future::LinkFuture;

enum Msg {
    ModifyProjectID(String),
    Error(String),
    GetProject,
    SetProject(project_view::ProjectViewProject),
}

struct Model {
    link: ComponentLink<Self>,
    project_id: String,
    project: project_view::ProjectViewProject,
}

impl Component for Model {
    type Message = Msg;

    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            project_id: String::new(),
            project: project_view::ProjectViewProject {
                id: "".into(),
                name: "".into(),
            },
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::GetProject => {
                let variables = project_view::Variables {
                    id: self.project_id.clone(),
                };
                self.link.send_future(async {
                    let api_client =
                        ApiClient::new(HttpClientImpl::new("http://localhost:8000".into()));
                    let response = api_client.send_query::<ProjectView>(variables).await;
                    match response {
                        Err(e) => Msg::Error(format!("{:?}", e)),
                        Ok(r) => Msg::SetProject(r.project),
                    }
                });
                true
            }
            Msg::SetProject(project) => {
                self.project = project;
                true
            }
            Msg::ModifyProjectID(s) => {
                self.project_id = s;
                true
            }
            Msg::Error(s) => {
                yew::services::DialogService::alert(&s);
                true
            }
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        // Should only return "true" if new properties are different to

        // previously received properties.

        // This component has no properties so we will always return "false".

        false
    }

    fn view(&self) -> Html {
        html! {
            <div>
                <div>
                    <input type="text" oninput=self.link.callback(|e: InputData| Msg::ModifyProjectID(e.value)) />
                    <button  onclick=self.link.callback(|_|Msg::GetProject)>{"送信"}</button>
                </div>
                <div>
                {"project_id:"} {&self.project.id}{"project_name:"} {&self.project.name}
                </div>
            </div>

        }
    }
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../../../../schema/graphql/api.graphql",
    query_path = "query/query.graphql",
    response_derives = "Debug"
)]
struct ProjectView;

fn main() {
    yew::start_app::<Model>();
}

#[cfg(test)]
mod tests {
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    fn my_async_test() {}
}
