#[derive(Debug)]
enum Statement {
	ClassItem(),
	FunctionItem(),
	VarDeclaration(),
	LetDeclaration(),
	ConstDeclaration()
}
